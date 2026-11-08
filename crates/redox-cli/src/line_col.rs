/// Resolve a byte offset to a 1-based line and column.
pub(crate) fn line_col(source: &str, offset: usize) -> (usize, usize) {
  let clamped = offset.min(source.len());
  let before = &source[..clamped];
  let line = before.matches('\n').count() + 1;
  let col = before
    .rsplit('\n')
    .next()
    .map_or(1, |l| l.chars().count() + 1);
  (line, col)
}

#[cfg(test)]
mod tests {
  use super::line_col;

  #[test]
  fn resolves_a_byte_offset_to_a_line_and_column() {
    let src = "main = ->\n  print \"x\"\n";
    assert_eq!(line_col(src, 0), (1, 1));
    assert_eq!(line_col(src, 12), (2, 3));
  }

  #[test]
  fn clamps_an_offset_past_the_end() {
    assert_eq!(line_col("ab", 99), (1, 3));
  }
}
