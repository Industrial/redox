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
  // `f ()` is doc 18 §7's replacement for doc 03 §5.2's `!` zero-argument
  // suffix, so a lone unit argument **is** the empty argument list — `f ()`
  // must become `f()` and not `f(())`. Juxtaposition needs something to
  // juxtapose, and `()` is that something.
  //
  // The trade is that a function genuinely taking a `()` cannot be called by
  // juxtaposition. Doc 18 §7 accepts it by replacing the suffix with this
  // spelling, and a unit parameter is vanishingly rare against a
  // zero-argument call.
  if matches!(args, [Expr::Unit { .. }]) {
    return String::new();
  }

  args
    .iter()
    .map(|e| emit_expr(unparen(e), indent))
    .collect::<Vec<_>>()
    .join(", ")
}
