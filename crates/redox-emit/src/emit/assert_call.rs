use crate::emit::expr::emit_expr;
use crate::emit::unparen::unparen;
use redox_ast::{BinaryOp, Expr};

/// Lower an `assert` statement — doc 03 §12.6, §15.
///
/// ```text
/// assert add 1 2 == 3     assert_eq!(add(1, 2), 3)
/// assert x != y       →   assert_ne!(x, y)
/// assert flag             assert!(flag)
/// ```
///
/// **`==` becomes `assert_eq!` and `!=` becomes `assert_ne!`**, which is not
/// decoration: those macros print both values when they differ and `assert!`
/// prints only the expression's text. A test that fails and says `assertion
/// failed: add(1, 2) == 3` has told the author nothing they did not type.
///
/// Decidable from the source text alone — doc 03 §2's admission test 1 — and
/// the only thing this looks at is the operator the author wrote.
pub(crate) fn assert_call(e: &Expr, indent: usize) -> String {
  if let Expr::Binary { op, lhs, rhs, .. } = unparen(e) {
    let macro_name = match op {
      BinaryOp::Eq => Some("assert_eq"),
      BinaryOp::Ne => Some("assert_ne"),
      _ => None,
    };
    if let Some(macro_name) = macro_name {
      return format!(
        "{macro_name}!({}, {})",
        emit_expr(unparen(lhs), indent),
        emit_expr(unparen(rhs), indent)
      );
    }
  }
  format!("assert!({})", emit_expr(unparen(e), indent))
}
