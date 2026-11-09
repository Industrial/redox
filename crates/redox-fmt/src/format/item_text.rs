/// One item's text with trailing whitespace stripped from every line — plan
/// 34, leaf 1.
///
/// The caller supplies a range already trimmed by `trimmed_range`, so this
/// only has the per-line work to do.
///
/// Trailing whitespace is the one thing rewritten inside an item. It is
/// invisible, never intentional, and cannot change meaning — the off-side rule
/// reads *leading* whitespace.
pub(crate) fn item_text(source: &str, range: (usize, usize)) -> String {
  source[range.0..range.1]
    .lines()
    .map(str::trim_end)
    .collect::<Vec<_>>()
    .join("\n")
}
