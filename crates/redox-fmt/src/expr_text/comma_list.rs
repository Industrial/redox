use crate::expr_text::expr_text;

use redox_ast::Expr;

/// A comma-separated run of expressions, with no trailing comma.
///
/// §3 row 12 is free here rather than implemented: the formatter prints from
/// the tree, and the tree has no record of a trailing comma. The one place a
/// trailing comma survives is `(a,)`, which is a one-element **tuple** and not
/// a group — plan 33 §10.3 — and that case is spelled out at the call site
/// rather than smuggled in here.
///
/// Each element is printed at power 0, because a comma is looser than every
/// operator.
pub(crate) fn comma_list(items: &[Expr], indent: usize, source: &str) -> String {
  let parts: Vec<String> = items
    .iter()
    .map(|item| expr_text(item, 0, indent, source))
    .collect();
  parts.join(", ")
}
