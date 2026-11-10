use crate::Span;

/// A character cursor over the source text.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 1. Replaces plan 01's
/// line loop, which could not carry bracket depth across a line boundary and
/// could not let one token span several lines.
///
/// The cursor tracks a byte offset and nothing else. It tracked a line and
/// column too until plan 05: those existed only to stamp a position onto a
/// [`crate::LexError`], and the UI corpus showed that a scanner running over a
/// *sub-slice* — a string's body — stamped a position relative to that slice
/// and so named the wrong line. Deriving line and column from the span in one
/// place (`redox_diag`'s `line_col`) removed both the duplicate computation
/// and the class of bug.
///
/// Every scanner takes `&mut Cursor` and nothing else, so each is testable
/// without constructing a whole [`super::Lexer`].
pub(crate) struct Cursor<'a> {
  /// The whole source text the offsets index into.
  src: &'a str,
  /// Byte offset of the next unconsumed character.
  pos: usize,
}

impl<'a> Cursor<'a> {
  /// A cursor at the start of a source.
  pub(crate) fn new(src: &'a str) -> Self {
    Self { src, pos: 0 }
  }

  /// The byte offset of the next unconsumed character.
  pub(crate) fn pos(&self) -> usize {
    self.pos
  }

  /// Whether the whole source has been consumed.
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

  /// The unconsumed text, for a scanner that matches on more than one
  /// character at a time.
  ///
  /// Doc 18 §5.1's comment rule needs it: whether `--` opens a comment
  /// depends on the character *after* the dashes, so the decision cannot be
  /// made with `starts_with` alone without enumerating every terminator.
  pub(crate) fn rest(&self) -> &'a str {
    &self.src[self.pos..]
  }

  /// Consume and return the next character.
  pub(crate) fn bump(&mut self) -> Option<char> {
    let ch = self.peek()?;
    self.pos += ch.len_utf8();
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

  /// The source text a span covers.
  pub(crate) fn slice(&self, span: Span) -> &'a str {
    &self.src[span.0..span.1]
  }

  /// Rewind to a previously recorded position.
  ///
  /// Used only for bounded lookahead — deciding whether `'` opens a character
  /// literal or a label — where scanning forward and backing out is clearer
  /// than a hand-rolled multi-character peek.
  pub(crate) fn reset_to(&mut self, pos: usize) {
    self.pos = pos;
  }

  /// The state [`Cursor::reset_to`] consumes.
  pub(crate) fn mark(&self) -> usize {
    self.pos
  }
}

#[cfg(test)]
mod tests {
  use super::Cursor;

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
  fn mark_and_reset_restore_the_position() {
    let mut c = Cursor::new("a\nb");
    let mark = c.mark();
    c.bump();
    c.bump();
    assert_eq!(c.pos(), 2);
    c.reset_to(mark);
    assert_eq!(c.pos(), 0);
  }

  /// A multi-byte character advances the offset by its encoded width, which is
  /// what keeps a span a valid slice range.
  #[test]
  fn bump_advances_by_the_encoded_width() {
    let mut c = Cursor::new("é x");
    assert_eq!(c.bump(), Some('é'));
    assert_eq!(c.pos(), 2);
  }
}
