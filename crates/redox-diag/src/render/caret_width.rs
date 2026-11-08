use crate::Span;

/// How many `^` to draw under a span.
///
/// Characters, not bytes. A span is a byte range, so subtracting its ends
/// gives the right number only for ASCII — plan 05's UI corpus caught it
/// drawing `^^` under a single `§`, because `§` is two bytes. The column the
/// caret starts at is already counted in characters, so the width has to be
/// too or the two disagree.
///
/// A span that is empty, or that does not line up with the text, still gets
/// one caret: a diagnostic with no visible marker is worse than one whose
/// marker is a character wide.
pub(crate) fn caret_width(source: &str, span: Span) -> usize {
  source
    .get(span.0..span.1)
    .map_or(1, |text| text.chars().count().max(1))
}

#[cfg(test)]
mod tests {
  use super::caret_width;

  #[test]
  fn counts_characters_not_bytes() {
    // `§` is two bytes but occupies one column.
    assert_eq!(caret_width("1 § 2", (2, 4)), 1);
  }

  #[test]
  fn spans_several_characters() {
    assert_eq!(caret_width("print 0x", (6, 8)), 2);
  }

  #[test]
  fn always_draws_at_least_one() {
    assert_eq!(caret_width("abc", (1, 1)), 1);
  }

  /// A span past the end — an error at end of input — must not panic.
  #[test]
  fn survives_a_span_outside_the_source() {
    assert_eq!(caret_width("ab", (5, 9)), 1);
  }

  /// Nor one that would slice through a character.
  #[test]
  fn survives_a_span_off_a_character_boundary() {
    assert_eq!(caret_width("é", (1, 2)), 1);
  }
}
