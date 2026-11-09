/// The leading whitespace for a given depth.
///
/// §3 row 11: **two spaces per level**, which is every `.sdx` in the corpus.
/// The one place a depth becomes characters, so the rule has exactly one
/// implementation — and so a future `rustfmt.toml`-style setting would have
/// one place to reach.
///
/// Doc 01 §3.1 said four spaces and plan 01 §4 recorded the discrepancy with
/// the corpus. The corpus won, per §1.5.
pub(crate) fn indent_text(indent: usize) -> String {
  "  ".repeat(indent)
}
