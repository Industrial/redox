use crate::Span;

/// Move a span onto a character that can actually be drawn under.
///
/// A parse error at end of input carries a span starting at `source.len()`,
/// and `miette` renders such a label as nothing at all: the snippet box
/// appears with no `┬` and no `here`. Rather than widen the label — which
/// would underline the wrong text — the span is walked back to the last
/// character that is not a line ending, so "unexpected end of input" points at
/// the last thing the author wrote.
///
/// A span already inside the text is returned untouched, so this costs nothing
/// in the common case.
pub(crate) fn anchor_span(source: &str, span: Span) -> Span {
  if span.0 < source.len() {
    return span;
  }
  let trimmed = source.trim_end_matches(['\n', '\r']);
  match trimmed.char_indices().next_back() {
    Some((i, c)) => (i, i + c.len_utf8()),
    // An empty source has nothing to point at; leave the span alone and let
    // the renderer print a box with no label.
    None => span,
  }
}

#[cfg(test)]
mod tests {
  use super::anchor_span;

  #[test]
  fn leaves_a_span_inside_the_text_alone() {
    assert_eq!(anchor_span("abc\n", (1, 2)), (1, 2));
  }

  #[test]
  fn walks_back_past_a_trailing_newline() {
    let src = "ab\n";
    assert_eq!(anchor_span(src, (3, 3)), (1, 2));
    assert_eq!(
      &src[1..2],
      "b",
      "the anchored span covers the last character"
    );
  }

  #[test]
  fn respects_character_boundaries() {
    // `é` is two bytes; a naive `len - 1` would slice mid-character.
    let src = "é\n";
    assert_eq!(anchor_span(src, (3, 3)), (0, 2));
    assert_eq!(&src[0..2], "é");
  }

  #[test]
  fn gives_up_on_an_empty_source() {
    assert_eq!(anchor_span("", (0, 0)), (0, 0));
    assert_eq!(anchor_span("\n\n", (2, 2)), (2, 2));
  }
}
