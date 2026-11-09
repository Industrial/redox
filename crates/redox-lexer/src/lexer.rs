pub(crate) mod cursor;
pub(crate) mod dedent_block;
pub(crate) mod is_ident_continue;
pub(crate) mod is_ident_start;
pub(crate) mod lex_fragment;
pub(crate) mod operator_table;
pub(crate) mod scan_byte;
pub(crate) mod scan_comment;
pub(crate) mod scan_cstring;
pub(crate) mod scan_escape;
pub(crate) mod scan_number;
pub(crate) mod scan_operator;
pub(crate) mod scan_quote;
pub(crate) mod scan_raw_ident;
pub(crate) mod scan_raw_string;
pub(crate) mod scan_string;
pub(crate) mod skip_interpolation;
pub(crate) mod split_format_spec;
pub(crate) mod trim_comment;

use crate::lexer::cursor::Cursor;
use crate::lexer::is_ident_continue::is_ident_continue;
use crate::lexer::is_ident_start::is_ident_start;
use crate::lexer::scan_byte::scan_byte;
use crate::lexer::scan_comment::scan_comment;
use crate::lexer::scan_cstring::scan_cstring;
use crate::lexer::scan_number::scan_number;
use crate::lexer::scan_operator::scan_operator;
use crate::lexer::scan_quote::scan_quote;
use crate::lexer::scan_raw_ident::scan_raw_ident;
use crate::lexer::scan_raw_string::scan_raw_string;
use crate::lexer::scan_string::scan_string;
use crate::lexer::trim_comment::trim_comment;
use crate::token::keyword::Keyword;
use crate::{LexError, Span, Token, TokenKind, Trivia, TriviaKind};
use redox_diag::Code;

/// The off-side-rule scanner.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`) replaced plan 01's line loop
/// with a [`Cursor`], because two features break the line-at-a-time shape
/// outright: continuation suppression needs bracket depth carried *across*
/// line boundaries, and a `"""` block is one token spanning several lines.
///
/// Private: [`crate::lex`] is the only way in. `Lexer` keeps the off-side rule
/// and orchestration; every scanner is a free function over `&mut Cursor`, so
/// each is testable without constructing a whole `Lexer`.
///
/// Indentation *width* is deliberately not fixed — any increase opens a block
/// and the matching decrease closes it, so two-space and four-space sources
/// both lex and only inconsistency is an error. Worth stating because doc 01
/// §3.1 prescribes four spaces while every example in docs 01–05 uses two; see
/// `docs/COMPILER.md` §4.
pub(crate) struct Lexer<'a> {
  cur: Cursor<'a>,
  toks: Vec<Token>,
  indents: Vec<usize>,
  pending: Vec<Trivia>,
  /// Nesting depth of `(`, `[`, `{`. While positive, newlines and indentation
  /// are not significant (plan 02 leaf 2).
  depth: usize,
  /// Byte offset of each currently-open delimiter, so an unbalanced one can be
  /// reported where it was opened rather than at `Eof`.
  open_delims: Vec<usize>,
  /// A comment that trailed content on the current line, held until the
  /// closing `Newline` exists to carry it.
  trailing: Option<Trivia>,
  /// Whether the cursor sits at the start of a **continuation** line.
  ///
  /// Plan 48 leaf 2. `scan_dashes` decides whether a comment leads the next
  /// token or trails the previous one by asking whether the last token is
  /// structural — which is its test for *"this line already has content"*,
  /// because at a line start the last token is a `Newline`, `Indent` or
  /// `Dedent`.
  ///
  /// Inside a continuation that test is **wrong**, and silently: the
  /// `Newline` is suppressed, so a comment opening the next line sees the
  /// previous line's last token — a `{`, say — decides it is a trailing
  /// comment, and is then held for a `Newline` that never arrives. The
  /// comment is dropped.
  ///
  /// Found by porting `redox-ast`'s `clause.rs`, whose record fields each
  /// carry a doc comment: every one vanished from the `.xrs`, and `redox fmt`
  /// deleted them from the `.sdx`. AGENTS.md calls comment preservation
  /// load-bearing — *"comments are trivia, never discarded"* (doc 02 §10.1)
  /// — and no corpus file had a comment inside a continuation, so nothing
  /// caught it.
  at_line_start: bool,
}

impl<'a> Lexer<'a> {
  pub(crate) fn new(src: &'a str) -> Self {
    Self {
      cur: Cursor::new(src),
      toks: Vec::new(),
      indents: vec![0],
      pending: Vec::new(),
      depth: 0,
      open_delims: Vec::new(),
      trailing: None,
      // The file's first line is a line start, and a comment opening the file
      // leads whatever follows it.
      at_line_start: true,
    }
  }

  /// Scan the whole source, consuming the lexer and yielding its tokens.
  pub(crate) fn run(mut self) -> Result<Vec<Token>, LexError> {
    // The first line's indentation opens the file's base level.
    self.start_of_line()?;
    while !self.cur.is_eof() {
      self.step()?;
    }
    self.finish()?;
    Ok(self.toks)
  }

  /// Handle one token, or the whitespace and trivia before it.
  fn step(&mut self) -> Result<(), LexError> {
    let ws = self.cur.eat_spaces() > 0;

    if self.cur.is_eof() {
      return Ok(());
    }

    if self.cur.peek() == Some('\n') {
      // A line ending in a binary operator or comma continues; and doc 03
      // §5.3 writes a pipeline with `|>` at the *start* of each continuation
      // line, so continuation is also a property of the line ahead.
      let trailing_continues = self.toks.last().is_some_and(|t| t.kind.continues_line());
      self.cur.bump();
      // Consuming the newline starts a line, on **every** path out of here —
      // which is the whole of plan 48 leaf 2's fix, and why it is set before
      // the branch rather than inside it.
      //
      // Setting it only on the continuation branch fixed the second field of
      // a record and not the first: after a `,` the continuation branch runs,
      // but after the opening `{` neither branch sets anything, because
      // `end_of_line` and `start_of_line` both return early while `depth > 0`
      // and no `Newline` token is produced for `scan_dashes` to see.
      self.at_line_start = true;
      if trailing_continues || self.next_line_continues() {
        // Consume the continuation line's indentation here, so the off-side
        // rule never sees it. Suppressing only the `Newline` would still let
        // `start_of_line` open a block — the bug this ordering fixes.
        self.cur.eat_spaces();
        return Ok(());
      }
      self.end_of_line();
      self.start_of_line()?;
      return Ok(());
    }

    // A '\r' from CRLF input is neither content nor indentation.
    if self.cur.peek() == Some('\r') {
      self.cur.bump();
      return Ok(());
    }

    if self.scan_dashes()? || self.scan_hash()? {
      return Ok(());
    }

    let start = self.cur.pos();
    let kind = self.scan_token()?;
    self.track_delims(&kind, start);
    self.push(kind, (start, self.cur.pos()), ws);
    Ok(())
  }

  /// Everything that is not whitespace, a newline or a `#` form.
  fn scan_token(&mut self) -> Result<TokenKind, LexError> {
    let ch = self.cur.peek().expect("step checked for eof");

    if ch == '"' {
      return scan_string(&mut self.cur);
    }
    if ch == '\'' {
      return scan_quote(&mut self.cur);
    }
    if ch.is_ascii_digit() {
      return scan_number(&mut self.cur);
    }

    // Prefixed literals, each only when the prefix is immediately followed by
    // its delimiter — otherwise `r`, `b` and `c` are ordinary identifiers.
    // A raw identifier `r#type` and a raw string `r#"…"#` share the first two
    // characters, and plan 23's raw string got the spelling first — so
    // `r#type` failed with *"a raw string must open with `\"`"* rather than
    // as an unknown construct (plan 45 §1.4). One more character of lookahead
    // separates them, which is what Rust's own lexer does.
    if ch == 'r' && self.cur.peek_at(1) == Some('#') {
      if self.cur.peek_at(2).is_some_and(|c| is_ident_start(c as u8)) {
        return scan_raw_ident(&mut self.cur);
      }
      return scan_raw_string(&mut self.cur);
    }
    if ch == 'r' && self.cur.peek_at(1) == Some('"') {
      return scan_raw_string(&mut self.cur);
    }
    if ch == 'b' && matches!(self.cur.peek_at(1), Some('"' | '\'')) {
      return scan_byte(&mut self.cur);
    }
    if ch == 'c' && self.cur.peek_at(1) == Some('"') {
      return scan_cstring(&mut self.cur);
    }

    if is_ident_start(ch as u8) {
      let span = self.cur.eat_while(|c| is_ident_continue(c as u8));
      let text = self.cur.slice(span);
      return Ok(match Keyword::from_text(text) {
        Some(kw) => TokenKind::Kw(kw),
        None => TokenKind::Ident(text.to_string()),
      });
    }

    if let Some(kind) = scan_operator(&mut self.cur) {
      return Ok(kind);
    }

    let start = self.cur.pos();
    self.cur.bump();
    Err(LexError {
      code: Code::UnexpectedCharacter,
      help: None,
      message: format!("unexpected character {ch:?}"),
      span: (start, self.cur.pos()),
    })
  }

  /// The `#` forms doc 18 §5.1 keeps: `#[attr]`, `#![attr]` and `#{`.
  ///
  /// Doc 03 §4.1 gave `#` six meanings and ordered six rules longest-first to
  /// tell them apart. Doc 18 moves comments to `--`, which retires four of
  /// them and leaves this function with the two that carry no comment
  /// meaning. A bare `#` is now an unexpected character rather than a line
  /// comment.
  ///
  /// Returns whether a `#` form was consumed.
  fn scan_hash(&mut self) -> Result<bool, LexError> {
    if self.cur.peek() != Some('#') {
      return Ok(false);
    }
    let start = self.cur.pos();

    if self.cur.starts_with("#[") || self.cur.starts_with("#![") {
      // An attribute, carried verbatim for passthrough (doc 03 §13.2).
      let inner = self.cur.starts_with("#![");
      self.cur.eat(if inner { "#![" } else { "#[" });
      let body_start = self.cur.pos();
      let mut depth = 1i32;
      while depth > 0 {
        let Some(ch) = self.cur.bump() else {
          return Err(LexError {
            code: Code::UnterminatedAttribute,
            help: None,
            message: "unterminated attribute; expected `]`".to_string(),
            span: (start, self.cur.pos()),
          });
        };
        match ch {
          '[' => depth += 1,
          ']' => depth -= 1,
          _ => {}
        }
      }
      let text = self.cur.slice((body_start, self.cur.pos() - 1)).to_string();
      // **Trivia, not a token** — plan 18 §1.3. An attribute belongs to the
      // declaration below it, which is what `leading` already means, and
      // every item kind and every statement carries one.
      self.pending.push(Trivia::comment(
        if inner {
          TriviaKind::InnerAttribute
        } else {
          TriviaKind::Attribute
        },
        text,
        (start, self.cur.pos()),
      ));
      return Ok(true);
    }

    Ok(false)
  }

  /// Doc 18 §5.1's comment forms: `--`, `-- |`, `-- !` and `{- … -}`.
  ///
  /// Returns whether a comment was consumed.
  fn scan_dashes(&mut self) -> Result<bool, LexError> {
    let start = self.cur.pos();

    // `{- … -}` is the fenced form, replacing doc 03's `### … ###`. It may
    // span lines and it nests, as Haskell's does — nesting is free here and a
    // non-nesting fence cannot comment out code that already has one.
    if self.cur.starts_with("{-") {
      self.cur.eat("{-");
      let body_start = self.cur.pos();
      let mut depth = 1i32;
      while depth > 0 {
        if self.cur.starts_with("{-") {
          self.cur.eat("{-");
          depth += 1;
          continue;
        }
        if self.cur.starts_with("-}") {
          self.cur.eat("-}");
          depth -= 1;
          continue;
        }
        if self.cur.bump().is_none() {
          return Err(LexError {
            code: Code::UnterminatedFence,
            help: Some("close it with `-}`".to_string()),
            message: "unterminated `{- -}` comment".to_string(),
            span: (start, self.cur.pos()),
          });
        }
      }
      let end = self.cur.pos() - 2;
      let text = self.cur.slice((body_start, end)).trim().to_string();
      self.pending.push(Trivia::comment(
        TriviaKind::FencedComment,
        text,
        (start, self.cur.pos()),
      ));
      return Ok(true);
    }

    let Some((kind, marker)) = scan_comment(self.cur.rest()) else {
      return Ok(false);
    };
    for _ in 0..marker {
      self.cur.bump();
    }

    let body = self.cur.eat_while(|c| c != '\n');
    let text = trim_comment(self.cur.slice(body));
    let trivium = Trivia::comment(kind, text, (start, self.cur.pos()));

    // A comment following content on the same line *trails* it and belongs to
    // the `Newline` that closes the line — the distinction plan 01 made, which
    // plan 02's rewrite had to restore. "The last token is non-structural" is
    // exactly the test for "this line already has content": at a line start
    // the last token is a `Newline`, `Indent` or `Dedent`, and at file start
    // there is none.
    // `at_line_start` is what makes this correct inside a continuation, where
    // the `Newline` that would otherwise answer the question is suppressed and
    // the test below would read the *previous* line's last token (plan 48
    // leaf 2).
    if !self.at_line_start && self.toks.last().is_some_and(|t| !t.kind.is_structural()) {
      self.trailing = Some(trivium);
    } else {
      self.pending.push(trivium);
    }
    Ok(true)
  }

  /// Maintain bracket depth, so continuation suppression knows when a
  /// newline is insignificant.
  fn track_delims(&mut self, kind: &TokenKind, start: usize) {
    match kind {
      TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
        self.depth += 1;
        self.open_delims.push(start);
      }
      TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
        self.depth = self.depth.saturating_sub(1);
        self.open_delims.pop();
      }
      _ => {}
    }
  }

  /// Whether the line ahead opens with a leading continuation operator.
  ///
  /// Only `|>` qualifies, and deliberately so. A leading `.` is ambiguous:
  /// doc 03 §7.5's `with` block uses it to introduce *statements*
  /// (`.insert "a", v`), so treating it as continuation would merge a whole
  /// block into one expression. `|>` has no such second meaning.
  fn next_line_continues(&mut self) -> bool {
    let mark = self.cur.mark();
    self.cur.eat_spaces();
    let continues = self.cur.starts_with("|>");
    self.cur.reset_to(mark);
    continues
  }

  /// Emit the statement separator closing a line, unless the line is
  /// continued (plan 02 leaf 2).
  fn end_of_line(&mut self) {
    if self.depth > 0 {
      return;
    }
    let continued = self.toks.last().is_some_and(|t| t.kind.continues_line());
    if continued {
      return;
    }
    if self.toks.last().is_some_and(|t| !t.kind.is_structural()) {
      let at = (self.cur.pos(), self.cur.pos());
      self.push(TokenKind::Newline, at, false);
      // Hand the line's trailing comment to the `Newline` that closes it.
      let trailing = self.trailing.take();
      self
        .toks
        .last_mut()
        .expect("a Newline was just pushed")
        .trailing = trailing;
    }
  }

  /// Apply the off-side rule at the start of a line.
  ///
  /// A blank or comment-only line leaves the indent stack untouched, because
  /// an indentation change is only ever signalled by a line with content.
  fn start_of_line(&mut self) -> Result<(), LexError> {
    if self.depth > 0 {
      return Ok(());
    }
    loop {
      let line_start = self.cur.pos();
      let width = self.measure_indent()?;

      match self.cur.peek() {
        None => return Ok(()),
        Some('\n') => {
          self.cur.bump();
          self.push_blank(line_start, self.cur.pos());
          continue;
        }
        Some('\r') => {
          self.cur.bump();
          continue;
        }
        // A comment-only line: consume it as trivia and look at the next.
        // `#` still reaches here for an attribute, which is content.
        Some('#' | '-' | '{') => {
          let before = self.cur.pos();
          if !self.scan_dashes()? && !self.scan_hash()? {
            return self.sync_indent(width, line_start);
          }
          // An attribute or set literal is content, not trivia, so the
          // indentation we just measured does apply to it.
          if self.toks.last().is_some_and(|t| t.span.0 >= before) {
            return self.sync_indent(width, line_start);
          }
          // A comment line's own terminating newline is not a blank line.
          // Consuming it here rather than letting the loop see it is what
          // stops two adjacent comments from gaining a blank line between
          // them in the generated Rust.
          if self.cur.peek() == Some('\n') {
            self.cur.bump();
          }
          continue;
        }
        Some(_) => return self.sync_indent(width, line_start),
      }
    }
  }

  /// Count leading spaces, rejecting tabs outright.
  ///
  /// Doc 01 §3.1 permits either rejection or one documented normalisation
  /// rule. Rejection is one branch and removes a class of indentation bug
  /// that otherwise reports against the wrong line.
  fn measure_indent(&mut self) -> Result<usize, LexError> {
    let mut width = 0usize;
    loop {
      match self.cur.peek() {
        Some(' ') => {
          self.cur.bump();
          width += 1;
        }
        Some('\t') => {
          let start = self.cur.pos();
          self.cur.bump();
          return Err(LexError {
            code: Code::TabIndent,
            help: Some("indent with spaces".to_string()),
            message: "tab in indentation; Redox indents with spaces".to_string(),
            span: (start, self.cur.pos()),
          });
        }
        _ => return Ok(width),
      }
    }
  }

  /// Reconcile a line's indentation against the indent stack, emitting
  /// `Indent` or one or more `Dedent`s.
  fn sync_indent(&mut self, width: usize, line_start: usize) -> Result<(), LexError> {
    let top = *self.indents.last().expect("indent stack is never empty");
    let at = (line_start + width, line_start + width);

    if width > top {
      self.indents.push(width);
      self.push(TokenKind::Indent, at, false);
      return Ok(());
    }

    while width < *self.indents.last().expect("indent stack is never empty") {
      self.indents.pop();
      self.push(TokenKind::Dedent, at, false);
    }
    if width != *self.indents.last().expect("indent stack is never empty") {
      return Err(LexError {
        code: Code::InconsistentDedent,
        help: Some("match an enclosing indentation level".to_string()),
        message: format!(
          "inconsistent dedent: {width} spaces matches no enclosing indentation level"
        ),
        span: at,
      });
    }
    Ok(())
  }

  /// Close every open block and emit `Eof`, which carries trailing trivia.
  fn finish(&mut self) -> Result<(), LexError> {
    if let Some(&pos) = self.open_delims.first() {
      return Err(LexError {
        code: Code::UnbalancedDelimiter,
        help: Some("add the closing delimiter".to_string()),
        message: "unbalanced delimiter: this is never closed".to_string(),
        span: (pos, pos + 1),
      });
    }

    self.end_of_line();
    let end = self.cur.pos();
    while self.indents.len() > 1 {
      self.indents.pop();
      self.push(TokenKind::Dedent, (end, end), false);
    }
    self.push(TokenKind::Eof, (end, end), false);
    Ok(())
  }

  /// Append a token, handing it any pending trivia unless it is structural —
  /// see [`TokenKind::is_structural`].
  fn push(&mut self, kind: TokenKind, span: Span, ws_before: bool) {
    // Content ends the line start, which is what makes a later comment on the
    // same line a *trailing* one.
    if !kind.is_structural() {
      self.at_line_start = false;
    }
    let leading = if kind.is_structural() {
      Vec::new()
    } else {
      std::mem::take(&mut self.pending)
    };
    self.toks.push(Token {
      kind,
      span,
      ws_before,
      leading,
      trailing: None,
    });
  }

  /// Record a blank line, merging it into a contiguous preceding run so the
  /// emitter sees one `BlankLines` per run rather than one per line.
  fn push_blank(&mut self, start: usize, end: usize) {
    match self.pending.last_mut() {
      Some(t) if t.kind == TriviaKind::BlankLines => t.span.1 = end,
      _ => self.pending.push(Trivia::blank((start, end))),
    }
  }
}
