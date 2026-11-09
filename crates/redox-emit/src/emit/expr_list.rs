use crate::emit::expr::emit_expr;
use crate::emit::unparen::unparen;
use redox_ast::Expr;

/// A comma-separated argument list, whose first line begins at `indent`.
///
/// Every argument is given the same column: a multi-line argument's
/// continuation lines line up with the call, which is what rustfmt does to a
/// list that fits on one line. A list that does *not* fit is `print_call`'s
/// problem, because `println!` is the only emitted call that can produce one.
pub(crate) fn expr_list(args: &[Expr], indent: usize) -> String {
  args
    .iter()
    .map(|e| emit_expr(unparen(e), indent))
    .collect::<Vec<_>>()
    .join(", ")
}
