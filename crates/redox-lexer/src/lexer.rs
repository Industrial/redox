pub(crate) mod cursor;
pub(crate) mod dedent_block;
pub(crate) mod is_ident_continue;
pub(crate) mod is_ident_start;
pub(crate) mod lex_fragment;
pub(crate) mod operator_table;
pub(crate) mod scan_byte;
pub(crate) mod scan_cstring;
pub(crate) mod scan_escape;
pub(crate) mod scan_number;
pub(crate) mod scan_operator;
pub(crate) mod scan_quote;
pub(crate) mod scan_raw_string;
pub(crate) mod scan_string;
pub(crate) mod split_format_spec;
pub(crate) mod trim_comment;

use crate::lexer::cursor::Cursor;
use crate::lexer::is_ident_continue::is_ident_continue;
use crate::lexer::is_ident_start::is_ident_start;
use crate::lexer::scan_byte::scan_byte;
use crate::lexer::scan_cstring::scan_cstring;
use crate::lexer::scan_number::scan_number;
use crate::lexer::scan_operator::scan_operator;
use crate::lexer::scan_quote::scan_quote;
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

    if self.scan_hash()? {
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
    if ch == 'r' && matches!(self.cur.peek_at(1), Some('"' | '#')) {
      return scan_raw_string(&mut self.cur);
    }
    if ch == 'b' && matches!(self.cur.peek_at(1), Some('"' | '\'')) {
      return scan_byte(&mut self.cur);
    }
    if ch == 'c' && self.cur.peek_at(1) == Some('"') {
      return scan_cstring(&mut self.cur);
    }

    // `%w[` — a word-array literal (doc 03 §10.4).
    if ch == '%' && self.cur.peek_at(1) == Some('w') && self.cur.peek_at(2) == Some('[') {
      self.cur.eat("%w[");
      return Ok(TokenKind::PercentW);
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

  /// Doc 03 §4.1's six `#` forms, by longest match in the documented order.
  ///
  /// Returns whether a `#` form was consumed. Comments become trivia; `#[`
  /// and `#{` are tokens.
  fn scan_hash(&mut self) -> Result<bool, LexError> {
    if self.cur.peek() != Some('#') {
      return Ok(false);
    }
    let start = self.cur.pos();

    // 1. `###` — a fenced comment, which may span lines.
    if self.cur.starts_with("###") {
      self.cur.eat("###");
      let body_start = self.cur.pos();
      while !self.cur.starts_with("###") {
        if self.cur.bump().is_none() {
          return Err(LexError {
            code: Code::UnterminatedFence,
            help: None,
            message: "unterminated `###` comment fence".to_string(),
            span: (start, self.cur.pos()),
          });
        }
      }
      let text = self
        .cur
        .slice((body_start, self.cur.pos()))
        .trim()
        .to_string();
      self.cur.eat("###");
      self.pending.push(Trivia::comment(
        TriviaKind::FencedComment,
        text,
        (start, self.cur.pos()),
      ));
      return Ok(true);
    }

    // 2-3. `##!` inner and `##` outer doc comments.
    let kind = if self.cur.starts_with("##!") {
      self.cur.eat("##!");
      TriviaKind::InnerDocComment
    } else if self.cur.starts_with("##") {
      self.cur.eat("##");
      TriviaKind::DocComment
    } else if self.cur.starts_with("#[") || self.cur.starts_with("#![") {
      // 4. An attribute, carried verbatim for passthrough (doc 03 §13.2).
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
      self.push(
        TokenKind::Attr { text, inner },
        (start, self.cur.pos()),
        false,
      );
      return Ok(true);
    } else if self.cur.starts_with("#{") {
      // 5. A set literal's opening delimiter.
      self.cur.eat("#{");
      self.depth += 1;
      self.open_delims.push(start);
      self.push(TokenKind::HashBrace, (start, self.cur.pos()), false);
      return Ok(true);
    } else {
      // 6. An ordinary line comment.
      self.cur.bump();
      TriviaKind::LineComment
    };

    let body = self.cur.eat_while(|c| c != '\n');
    let text = trim_comment(self.cur.slice(body));
    let trivium = Trivia::comment(kind, text, (start, self.cur.pos()));

    // A comment following content on the same line *trails* it and belongs to
    // the `Newline` that closes the line — the distinction plan 01 made, which
    // this rewrite had to restore. "The last token is non-structural" is
    // exactly the test for "this line already has content": at a line start
    // the last token is a `Newline`, `Indent` or `Dedent`, and at file start
    // there is none.
    if self.toks.last().is_some_and(|t| !t.kind.is_structural()) {
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
      TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace | TokenKind::PercentW => {
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
        Some('#') => {
          let before = self.cur.pos();
          self.scan_hash()?;
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
