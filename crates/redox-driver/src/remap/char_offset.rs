/// Byte offset of a 1-based line and column, or `None` if the position is not
/// in the text.
///
/// Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) leaf 3. A column past the end
/// of its line is **not** a position in it, and returning `None` rather than
/// the line's end is what lets [`super::narrow`] tell the two cases apart: a
/// column that landed somewhere real, and one that the emitter's restructuring
/// has made meaningless.
pub(crate) fn char_offset(source: &str, line: usize, col: usize) -> Option<usize> {
  if line == 0 || col == 0 {
    return None;
  }
  let mut offset = 0;
  for (n, text) in source.lines().enumerate() {
    if n + 1 == line {
      if text.chars().count() < col - 1 {
        return None;
      }
      let within: usize = text.chars().take(col - 1).map(char::len_utf8).sum();
      return Some(offset + within);
    }
    offset += text.len() + 1;
  }
  None
}

#[cfg(test)]
mod tests {
  use super::char_offset;

  const SRC: &str = "ab\n  cd\n";

  #[test]
  fn resolves_a_position() {
    assert_eq!(char_offset(SRC, 1, 1), Some(0));
    assert_eq!(char_offset(SRC, 2, 3), Some(5));
    assert_eq!(&SRC[5..7], "cd");
  }

  /// One past the last character is a valid insertion point.
  #[test]
  fn allows_the_position_just_past_a_line() {
    assert_eq!(char_offset(SRC, 1, 3), Some(2));
  }

  #[test]
  fn declines_a_column_past_the_line() {
    assert_eq!(char_offset(SRC, 1, 9), None);
  }

  #[test]
  fn declines_a_line_past_the_text() {
    assert_eq!(char_offset(SRC, 9, 1), None);
  }

  #[test]
  fn declines_a_zero_line_or_column() {
    assert_eq!(char_offset(SRC, 0, 1), None);
    assert_eq!(char_offset(SRC, 1, 0), None);
  }

  /// Columns are counted in characters and offsets in bytes, so a multi-byte
  /// character must not make the result slice through one.
  #[test]
  fn counts_characters_and_returns_bytes() {
    assert_eq!(char_offset("é x\n", 1, 2), Some(2));
    assert_eq!(&"é x\n"[2..3], " ");
  }
}
