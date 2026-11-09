use crate::check_command::remap::char_offset::char_offset;
use redox_diag::Span;

/// Narrow `span` to the single character at `line`:`col`, if that position
/// falls inside it.
///
/// Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) §2.1 — the decision this
/// function exists to make.
///
/// Inside the span, rustc's column survives the shift intact and the caret can
/// be precise: `a + "x"` is emitted as `a + "x"`, so the only difference is
/// indentation. Outside it, the emitter restructured the line —
/// `print add("x", 2)` became `println!("{}", add("x", 2));` — and the shifted
/// column means nothing. Then the whole span is returned, and the reader sees
/// the statement underlined.
///
/// The rejected alternative was to clamp the column into the line. A caret that
/// is plausible and wrong is worse than one that is honestly coarse, because
/// the reader has no way to tell which they are looking at.
pub(crate) fn narrow(span: Span, source: &str, line: usize, col: usize) -> Span {
  let Some(offset) = char_offset(source, line, col) else {
    return span;
  };
  if offset < span.0 || offset >= span.1 {
    return span;
  }
  let width = source[offset..].chars().next().map_or(1, char::len_utf8);
  (offset, offset + width)
}

#[cfg(test)]
mod tests {
  use super::narrow;

  /// `add = …\n  a + "x"\n` — the statement spans bytes 33..40.
  const SRC: &str = "add = (a: i32, b: i32): i32 ->\n  a + \"x\"\n";
  const SPAN: (usize, usize) = (33, 40);

  #[test]
  fn narrows_to_one_character_inside_the_span() {
    let got = narrow(SPAN, SRC, 2, 5);
    assert_eq!(&SRC[got.0..got.1], "+");
  }

  #[test]
  fn keeps_the_span_when_the_column_is_past_the_line() {
    assert_eq!(narrow(SPAN, SRC, 2, 99), SPAN);
  }

  /// A position on another line is outside the span, even though it exists.
  #[test]
  fn keeps_the_span_when_the_position_is_elsewhere() {
    assert_eq!(narrow(SPAN, SRC, 1, 1), SPAN);
  }

  #[test]
  fn keeps_the_span_when_the_line_does_not_exist() {
    assert_eq!(narrow(SPAN, SRC, 99, 1), SPAN);
  }

  /// The narrowed span must cover a whole character, not a byte of one.
  #[test]
  fn narrows_to_a_whole_multibyte_character() {
    // `x :: ()\n` is 8 bytes and `x =\n` is 4, so the body's two spaces are
    // at 12 and 13 and `é` — two bytes — is 14..16, on line 3 column 3.
    let src = "x :: ()\nx =\n  é\n";
    let span = (14, 16);
    assert_eq!(&src[span.0..span.1], "é", "the premise of this test");
    let got = narrow(span, src, 3, 3);
    assert_eq!(&src[got.0..got.1], "é");
  }
}
