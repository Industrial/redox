use crate::stmt_text::indent_text::indent_text;

/// A `rust` escape hatch, printed in the layout spelling — plan 64 leaf 4.
///
/// ```text
/// markup = rust
///   html! { <div class="x">{ name }</div> }
/// ```
///
/// **One spelling out for two spellings in**, and that is the decision this
/// function carries. The `"""` form plan 45 shipped and the layout form plan
/// 64 adds reach the tree as the same node holding the same dedented text, so
/// the formatter has to pick one — and the layout form is the one §3's
/// canonical table can state a rule for. A `"""` block's closing delimiter
/// has no canonical column, so printing that spelling would mean the
/// formatter choosing one and then rewriting every hatch that disagreed.
///
/// Rewriting one spelling into the other is a **refactor and not a
/// normalisation** — the same line plan 34 §10.11 drew for the two pipeline
/// rewrites — so it is recorded here rather than left to read off the code:
/// the body is byte-identical either way, and only its delimiters change.
pub(crate) fn rust_hatch_text(code: &str, indent: usize) -> String {
  let pad = indent_text(indent + 1);
  let mut out = String::from("rust");
  for line in code.lines() {
    out.push('\n');
    if !line.trim().is_empty() {
      out.push_str(&pad);
      out.push_str(line);
    }
  }
  out
}
