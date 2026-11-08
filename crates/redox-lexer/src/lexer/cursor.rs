use crate::Span;

/// A character cursor over the source text, tracking its own line number.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 1. Replaces plan 01's
/// line loop, which could not carry bracket depth across a line boundary and
/// could not let one token span several lines.
///
/// Line numbers advance inside [`Cursor::bump`], so a multi-line token — a
/// `"""` block — cannot desynchronise them. Emerald needed a 447-line
/// preprocessing pass to get that property into a lexer whose shape could not
/// accommodate it.
///
/// Every scanner takes `&mut Cursor` and nothing else, so each is testable
/// without constructing a whole [`super::Lexer`].
pub(crate) struct Cursor<'a> {
  src: &'a str,
  /// Byte offset of the next unconsumed character.
  pos: usize,
  /// 1-based line of `pos`.
  line: usize,
  /// Byte offset at which the current line began.
  line_start: usize,
}

impl<'a> Cursor<'a> {
  pub(crate) fn new(src: &'a str) -> Self {
    Self {
      src,
      pos: 0,
      line: 1,
      line_start: 0,
    }
  }

  pub(crate) fn pos(&self) -> usize {
    self.pos
  }

  pub(crate) fn line(&self) -> usize {
    self.line
  }

  /// 1-based column of `pos` within its line, counted in characters.
  pub(crate) fn col(&self) -> usize {
    self.src[self.line_start..self.pos].chars().count() + 1
  }

  pub(crate) fn is_eof(&self) -> bool {
    self.pos >= self.src.len()
  }

  /// The next character without consuming it.
  pub(crate) fn peek(&self) -> Option<char> {
    self.src[self.pos..].chars().next()
  }

  /// The character `n` positions ahead without consuming anything.
  pub(crate) fn peek_at(&self, n: usize) -> Option<char> {
    self.src[self.pos..].chars().nth(n)
  }

  /// Whether the text at the cursor begins with `s`.
  pub(crate) fn starts_with(&self, s: &str) -> bool {
    self.src[self.pos..].starts_with(s)
  }

  /// Consume and return the next character.
  pub(crate) fn bump(&mut self) -> Option<char> {
    let ch = self.peek()?;
    self.pos += ch.len_utf8();
    if ch == '\n' {
      self.line += 1;
      self.line_start = self.pos;
    }
    Some(ch)
  }

  /// Consume `s` if it is at the cursor, reporting whether it was.
  pub(crate) fn eat(&mut self, s: &str) -> bool {
    if !self.starts_with(s) {
      return false;
    }
    for _ in 0..s.chars().count() {
      self.bump();
    }
    true
  }

  /// Consume characters while `pred` holds, returning the consumed span.
  pub(crate) fn eat_while(&mut self, mut pred: impl FnMut(char) -> bool) -> Span {
    let start = self.pos;
    while self.peek().is_some_and(&mut pred) {
      self.bump();
    }
    (start, self.pos)
  }

  /// Consume spaces, reporting how many. Does not cross a line boundary.
  pub(crate) fn eat_spaces(&mut self) -> usize {
    let mut n = 0;
    while self.peek() == Some(' ') {
      self.bump();
      n += 1;
    }
    n
  }

  pub(crate) fn slice(&self, span: Span) -> &'a str {
    &self.src[span.0..span.1]
  }

  /// Rewind to a previously recorded position.
  ///
  /// Used only for bounded lookahead — deciding whether `'` opens a character
  /// literal or a label — where scanning forward and backing out is clearer
  /// than a hand-rolled multi-character peek.
  pub(crate) fn reset_to(&mut self, pos: usize, line: usize, line_start: usize) {
    self.pos = pos;
    self.line = line;
    self.line_start = line_start;
  }

  /// The state `reset_to` consumes.
  pub(crate) fn mark(&self) -> (usize, usize, usize) {
    (self.pos, self.line, self.line_start)
  }
}

#[cfg(test)]
mod tests {
  use super::Cursor;

  #[test]
  fn tracks_lines_across_bumps() {
    let mut c = Cursor::new("ab\ncd");
    assert_eq!(c.line(), 1);
    c.bump();
    c.bump();
    assert_eq!(c.line(), 1);
    c.bump(); // the newline
    assert_eq!(c.line(), 2);
    assert_eq!(c.col(), 1);
    c.bump();
    assert_eq!(c.col(), 2);
  }

  #[test]
  fn eat_matches_only_an_exact_prefix() {
    let mut c = Cursor::new("...x");
    assert!(!c.eat("..="));
    assert!(c.eat("..."));
    assert_eq!(c.peek(), Some('x'));
  }

  #[test]
  fn eat_while_returns_the_consumed_span() {
    let mut c = Cursor::new("abc123");
    let span = c.eat_while(|ch| ch.is_ascii_alphabetic());
    assert_eq!(span, (0, 3));
    assert_eq!(c.slice(span), "abc");
  }

  #[test]
  fn mark_and_reset_restore_line_tracking() {
    let mut c = Cursor::new("a\nb");
    let mark = c.mark();
    c.bump();
    c.bump();
    assert_eq!(c.line(), 2);
    c.reset_to(mark.0, mark.1, mark.2);
    assert_eq!(c.line(), 1);
    assert_eq!(c.pos(), 0);
  }

  #[test]
  fn counts_columns_in_characters_not_bytes() {
    let mut c = Cursor::new("é x");
    c.bump();
    assert_eq!(c.col(), 2, "a two-byte char advances one column");
  }
}
