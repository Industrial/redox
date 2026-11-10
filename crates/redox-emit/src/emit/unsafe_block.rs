use crate::emit::emit_do::emit_do;
use crate::emit::expr::emit_expr;
use redox_ast::Block;

/// Emit Rust's `unsafe { … }` — plan 45 leaf 2.
///
/// `unsafe` in front of the block `do` already emits, with one refinement
/// that is not cosmetic: a body of **one expression** is emitted on a single
/// line.
///
/// rustfmt collapses `unsafe {\n    w.halves\n}` to `unsafe { w.halves }`, and
/// `generated_rust_is_rustfmt_clean` compares the emitter's output against
/// rustfmt's byte for byte — so emitting the long form for a one-expression
/// body fails the gate. Found exactly that way, by `examples/reachability.sdx`.
///
/// Only the one-expression case is collapsed. Guessing rustfmt's width rule
/// for anything longer would be a second formatter living in the emitter, and
/// the gate would catch each wrong guess one example at a time.
pub(crate) fn emit_unsafe_block(body: &Block, indent: usize) -> String {
  if let Some(e) = body.sole_expression() {
    return format!("unsafe {{ {} }}", emit_expr(e, indent));
  }
  format!("unsafe {}", emit_do(&None, body, indent))
}
