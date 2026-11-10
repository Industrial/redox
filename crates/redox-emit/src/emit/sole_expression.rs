use crate::emit::expr::emit_expr;
use crate::emit::unparen::unparen;
use redox_ast::Guard;

/// A guard row's body as one expression's text, if that is what it is.
///
/// A guard's body is parsed as a block of one tail statement, so this is
/// almost always `Some` — but "almost always" is not "always", and the
/// one-line `if`/`else` form has nowhere to put a second statement.
pub(crate) fn sole_expression(guard: &Guard, indent: usize) -> Option<String> {
  guard
    .body
    .sole_expression()
    .map(|e| emit_expr(unparen(e), indent))
}
