use crate::emit::expr::emit_expr;
use redox_ast::Expr;

/// A comma-separated argument list.
pub(crate) fn expr_list(args: &[Expr]) -> String {
  args.iter().map(emit_expr).collect::<Vec<_>>().join(", ")
}
