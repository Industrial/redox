use crate::emit::expr::emit_expr;
use redox_ast::Expr;

/// A pipeline callee's text, parenthesised when it is not a name.
///
/// Plan 58 (`docs/52 - 58 - The Reverse Pipeline.md`). A composition chain
/// lowers to a closure, and `|x| …(5)` parses as a closure whose body is a
/// call rather than as a call of a closure — so the closure needs wrapping
/// before the piped value can be applied to it.
///
/// A bare name or a path needs no parentheses and must not get any: the
/// goldens are asserted rustfmt-clean and rustfmt does not remove a redundant
/// pair around a call's callee.
pub(crate) fn callable(callee: &Expr, indent: usize) -> String {
  match callee {
    Expr::Ident(_) | Expr::Path { .. } => emit_expr(callee, indent),
    _ => format!("({})", emit_expr(callee, indent)),
  }
}
