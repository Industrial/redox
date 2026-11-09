use crate::emit::expr::emit_expr;
use crate::emit::unparen::unparen;
use redox_ast::Expr;

/// A comma-separated argument list.
pub(crate) fn expr_list(args: &[Expr]) -> String {
  args
    .iter()
    .map(|e| emit_expr(unparen(e)))
    .collect::<Vec<_>>()
    .join(", ")
}
