/// Resolve a byte offset to a 1-based line and column.
///
/// Columns are counted in characters, not bytes, so a diagnostic under a
/// multi-byte character points at the right place.
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
  fn resolves_a_byte_offset() {
    let src = "main : ()\nmain =\n  println! \\-> \"#{1}\"\n";
    assert_eq!(line_col(src, 0), (1, 1));
    // `main : ()\n` is 10 bytes, `main =\n` is 7, so the `p` of `print` is
    // at 19 — line 3, column 3.
    //
    // Both numbers moved by one when plan 52 made the type marker `:` rather
    // than `::`. The slice assertion below is deliberately there to catch
    // exactly that: an offset derived in a comment is only as trustworthy as
    // the assertion that re-derives it from the source.
    assert_eq!(&src[19..20], "p");
    assert_eq!(line_col(src, 19), (3, 3));
  }

  #[test]
  fn counts_columns_in_characters_not_bytes() {
    // `é` is two bytes, so a byte-based column would report 3.
    assert_eq!(line_col("é x", 3), (1, 3));
  }

  #[test]
  fn clamps_past_the_end() {
    assert_eq!(line_col("ab", 99), (1, 3));
  }
}
