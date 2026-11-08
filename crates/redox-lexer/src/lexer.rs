pub(crate) mod is_ident_continue;
pub(crate) mod is_ident_start;
pub(crate) mod trim_comment;

use crate::lexer::is_ident_continue::is_ident_continue;
use crate::lexer::is_ident_start::is_ident_start;
use crate::lexer::trim_comment::trim_comment;
use crate::{LexError, Span, Token, TokenKind, Trivia, TriviaKind};

/// The off-side-rule scanner.
///
/// Private: [`crate::lex`] is the only way in. Indentation *width* is
/// deliberately not fixed — any increase opens a block and the matching
/// decrease closes it, so two-space and four-space sources both lex and only
/// inconsistency is an error. Worth stating because doc 01 §3.1 prescribes
/// four spaces while every example in docs 01–05 uses two; see
/// `docs/COMPILER.md` §4. Enforcing one canonical width is a formatter
/// concern (plan 34), not a lexer one.
pub(crate) struct Lexer<'a> {
  src: &'a str,
  toks: Vec<Token>,
  indents: Vec<usize>,
  pending: Vec<Trivia>,
  line: usize,
}

impl<'a> Lexer<'a> {
  pub(crate) fn new(src: &'a str) -> Self {
    Self {
      src,
      toks: Vec::new(),
      indents: vec![0],
      pending: Vec::new(),
      line: 0,
    }
  }

  /// Scan the whole source, consuming the lexer and yielding its tokens.
  pub(crate) fn run(mut self) -> Result<Vec<Token>, LexError> {
    let mut offset = 0usize;
    for raw_line in self.src.split('\n') {
      self.line += 1;
      let line_start = offset;
      // `split('\n')` drops the separator; advance past the line and it.
      offset += raw_line.len() + 1;
      // A trailing '\r' from CRLF input is not indentation and not content.
      let text = raw_line.strip_suffix('\r').unwrap_or(raw_line);
      self.line_pass(text, line_start)?;
    }
    self.finish();
    Ok(self.toks)
  }

  /// Handle one physical line: indentation, then its tokens.
  fn line_pass(&mut self, text: &str, line_start: usize) -> Result<(), LexError> {
    let width = self.measure_indent(text, line_start)?;
    let body = &text[width..];

    // Blank line: trivia, and the indent stack is untouched. An indentation
    // change is only ever signalled by a line with content, so a blank line
    // inside a block does not close it.
    if body.is_empty() {
      self.push_blank(line_start, line_start + text.len());
      return Ok(());
    }

    // Comment-only line: trivia, indent stack untouched for the same reason.
    if let Some(rest) = body.strip_prefix('#') {
      let span = (line_start + width, line_start + text.len());
      self.pending.push(Trivia::comment(trim_comment(rest), span));
      return Ok(());
    }

    self.sync_indent(width, line_start)?;
    self.tokenize(body, line_start + width)?;
    Ok(())
  }

  /// Count leading spaces, rejecting tabs outright.
  ///
  /// Doc 01 §3.1 permits either rejection or one documented normalisation
  /// rule. Rejection is one branch and removes a class of indentation bug
  /// that otherwise reports against the wrong line.
  fn measure_indent(&self, text: &str, line_start: usize) -> Result<usize, LexError> {
    let mut width = 0usize;
    for (i, ch) in text.char_indices() {
      match ch {
        ' ' => width += 1,
        '\t' => {
          return Err(LexError {
            message: "tab in indentation; Redox indents with spaces".to_string(),
            span: (line_start + i, line_start + i + 1),
            line: self.line,
            col: i + 1,
          });
        }
        _ => break,
      }
    }
    Ok(width)
  }

  /// Reconcile this line's indentation against the indent stack, emitting
  /// `Indent`, `Newline` or one or more `Dedent`s.
  fn sync_indent(&mut self, width: usize, line_start: usize) -> Result<(), LexError> {
    let top = *self.indents.last().expect("indent stack is never empty");
    let at = (line_start + width, line_start + width);

    if width > top {
      self.indents.push(width);
      self.push(TokenKind::Indent, at, false);
      return Ok(());
    }

    if width == top {
      // A statement separator, but only between statements — not before the
      // first token in the file.
      if !self.toks.is_empty() {
        self.push(TokenKind::Newline, at, false);
      }
      return Ok(());
    }

    while width < *self.indents.last().expect("indent stack is never empty") {
      self.indents.pop();
      self.push(TokenKind::Dedent, at, false);
    }
    if width != *self.indents.last().expect("indent stack is never empty") {
      return Err(LexError {
        message: format!(
          "inconsistent dedent: {width} spaces matches no enclosing indentation level"
        ),
        span: at,
        line: self.line,
        col: width + 1,
      });
    }
    self.push(TokenKind::Newline, at, false);
    Ok(())
  }

  /// Tokenize one line's content, whose indentation is already consumed.
  /// Emits a closing `Newline`.
  fn tokenize(&mut self, body: &str, base: usize) -> Result<(), LexError> {
    let bytes = body.as_bytes();
    let mut i = 0usize;
    let mut ws = false;
    let mut trailing = None;

    while i < bytes.len() {
      let ch = bytes[i];
      if ch == b' ' {
        i += 1;
        ws = true;
        continue;
      }

      // A `#` after content on the same line: the rest of the line is trivia.
      if ch == b'#' {
        let span = (base + i, base + body.len());
        trailing = Some(Trivia::comment(trim_comment(&body[i + 1..]), span));
        break;
      }

      let start = base + i;

      if ch == b'"' {
        let mut j = i + 1;
        while j < bytes.len() && bytes[j] != b'"' {
          j += 1;
        }
        if j >= bytes.len() {
          return Err(LexError {
            message: "unterminated string literal".to_string(),
            span: (start, base + body.len()),
            line: self.line,
            col: i + 1,
          });
        }
        let raw = body[i..=j].to_string();
        self.push(TokenKind::StrLit(raw), (start, base + j + 1), ws);
        i = j + 1;
        ws = false;
        continue;
      }

      if ch == b'-' && bytes.get(i + 1) == Some(&b'>') {
        self.push(TokenKind::Arrow, (start, start + 2), ws);
        i += 2;
        ws = false;
        continue;
      }

      if ch == b'=' {
        self.push(TokenKind::Eq, (start, start + 1), ws);
        i += 1;
        ws = false;
        continue;
      }

      if is_ident_start(ch) {
        let mut j = i;
        while j < bytes.len() && is_ident_continue(bytes[j]) {
          j += 1;
        }
        let text = body[i..j].to_string();
        self.push(TokenKind::Ident(text), (start, base + j), ws);
        i = j;
        ws = false;
        continue;
      }

      return Err(LexError {
        message: format!("unexpected character {:?}", ch as char),
        span: (start, start + 1),
        line: self.line,
        col: i + 1,
      });
    }

    let end = base + body.len();
    self.push(TokenKind::Newline, (end, end), false);
    if let Some(t) = trailing {
      self
        .toks
        .last_mut()
        .expect("a Newline was just pushed")
        .trailing = Some(t);
    }
    Ok(())
  }

  /// Close every open block and emit `Eof`, which carries trailing trivia.
  fn finish(&mut self) {
    let end = self.src.len();
    while self.indents.len() > 1 {
      self.indents.pop();
      self.push(TokenKind::Dedent, (end, end), false);
    }
    self.push(TokenKind::Eof, (end, end), false);
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
