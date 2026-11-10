/// One byte offset as a 1-based `(line, column)` in **characters**.
///
/// Plan 50 §1.3. `redox-lsp`'s `position_of` answers the same question in the
/// unit LSP requires — 0-based lines and UTF-16 code units — and is therefore
/// the wrong function here rather than a reusable one. An MCP client reads a
/// file and quotes a line; it counts characters and starts at one, like every
/// editor's status bar and every compiler's error message.
///
/// An offset past the end clamps to the last position rather than panicking: a
/// diagnostic is not worth losing over an arithmetic edge, and a span one byte
/// past the end is what a *"something was expected here"* error legitimately
/// carries.
pub(crate) fn diagnostic_line_and_column(source: &str, offset: usize) -> (usize, usize) {
  let offset = offset.min(source.len());
  // Count the newlines before the offset; the last one ends the preceding
  // line, so the line after it is the one the offset is on.
  let line_start = source[..offset]
    .rfind('\n')
    .map_or(0, |at| at + '\n'.len_utf8());
  let line = source[..offset].matches('\n').count() + 1;
  // Characters, not bytes. `chars().count()` over the line prefix is the
  // whole of the difference from a byte column, and the reason this function
  // exists at all.
  let column = source[line_start..offset].chars().count() + 1;
  (line, column)
}

#[cfg(test)]
mod tests {
  use super::diagnostic_line_and_column;

  #[test]
  fn the_first_byte_is_one_one() {
    assert_eq!(diagnostic_line_and_column("abc", 0), (1, 1));
  }

  #[test]
  fn a_later_line_counts_from_its_own_start() {
    let source = "main : ()\nmain =\n  print 1\n";
    let at = source.find("print").expect("should be there");
    assert_eq!(diagnostic_line_and_column(source, at), (3, 3));
  }

  /// The property the whole function exists for: a multi-byte character is
  /// **one** column, not two or three.
  #[test]
  fn a_column_counts_characters_and_not_bytes() {
    let source = "-- é\nx";
    let at = source.find('x').expect("should be there");
    assert_eq!(diagnostic_line_and_column(source, at), (2, 1));

    // And within the line: `é` is two bytes, so a byte column would say 5.
    let line = "aéb";
    let at = line.find('b').expect("should be there");
    assert_eq!(at, 3, "the byte offset is 3");
    assert_eq!(diagnostic_line_and_column(line, at), (1, 3));
  }

  #[test]
  fn an_offset_past_the_end_clamps() {
    let source = "ab\n";
    assert_eq!(diagnostic_line_and_column(source, 9_999), (2, 1));
  }
}
