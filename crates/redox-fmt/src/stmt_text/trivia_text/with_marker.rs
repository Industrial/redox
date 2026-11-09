/// A comment line, with no trailing space when the comment is empty.
///
/// `--` on its own is a divider the corpus uses — `comments.sdx` and most
/// plan docs open with one — and `-- ` with nothing after it is trailing
/// whitespace, which leaf 1 established this formatter removes.
pub(crate) fn with_marker(pad: &str, marker: &str, text: &str) -> String {
  if text.is_empty() {
    format!("{pad}{marker}")
  } else {
    format!("{pad}{marker} {text}")
  }
}
