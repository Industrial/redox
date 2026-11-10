/// Choose the source line to show under a diagnostic, and where to point.
///
/// Returns `(line number, line text, column)`, all 1-based.
///
/// The subtlety is end-of-file. A file ending in a newline makes the offset
/// one past the last character resolve to a line that exists but holds no
/// text, and an "unexpected end of file" error rendered with no snippet at
/// all is the least useful diagnostic there is. So when the resolved line is
/// empty and there is a line above it, this backs up and points just past the
/// end of that line — which is where the reader has to type.
pub(crate) fn snippet_line(source: &str, line: usize, col: usize) -> Option<(usize, &str, usize)> {
  let text = source.lines().nth(line - 1).unwrap_or("");
  if !text.is_empty() {
    return Some((line, text, col));
  }
  let mut n = line;
  while n > 1 {
    n -= 1;
    let above = source.lines().nth(n - 1)?;
    if !above.is_empty() {
      return Some((n, above, above.chars().count() + 1));
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::snippet_line;

  #[test]
  fn keeps_a_line_that_has_text() {
    let src = "main : ()\nmain =\n  println! \\-> \"#{1}\"\n";
    assert_eq!(
      snippet_line(src, 3, 3),
      Some((3, "  println! \\-> \"#{1}\"", 3))
    );
  }

  /// The case that motivates the function: an error at end of file.
  #[test]
  fn backs_up_from_the_phantom_line_after_a_trailing_newline() {
    let src = "main : ()\nmain =\n  println! \\-> \"#{(1 +)}\"\n";
    // 26, not 14: the column is one past the line's last character, and
    // plan 64's `println! \-> "#{…}"` is twelve characters longer than the
    // `print …` it replaced.
    assert_eq!(
      snippet_line(src, 4, 1),
      Some((3, "  println! \\-> \"#{(1 +)}\"", 26))
    );
  }

  #[test]
  fn skips_blank_lines_on_the_way_up() {
    let src = "x\n\n\n";
    assert_eq!(snippet_line(src, 4, 1), Some((1, "x", 2)));
  }

  #[test]
  fn gives_up_on_an_empty_source() {
    assert_eq!(snippet_line("", 1, 1), None);
  }
}
