use crate::emit::emit_control::emit_control;
use crate::emit::sink::Sink;
use redox_ast::Expr;

/// A control-flow form's Rust as **text**, for an expression position.
///
/// `emit_control` writes into the caller's sink because a control-flow
/// statement spans lines; an expression's lowering is a `String`. So this is
/// the same lowering run into a sink of its own, with the two things that
/// make it a *statement* taken back off: the leading pad to `indent`, which
/// an expression's caller has already written, and the trailing newline.
///
/// A nested `Sink`, for the reason `emit_do` records — the records it collects
/// are discarded with it, so a loop reached through an expression position is
/// attributed to the expression that holds it rather than line by line. The
/// same coarseness, closed by the same change `emit_do`'s comment names.
///
/// Reached only by the indented `if`, `while`, `for` and `loop`, and only from
/// an expression position — which today's grammar cannot produce, since
/// `prefix` has no production for any of them. It is written rather than left
/// as a `todo!()` because an AST node with no lowering is a delay fuse, and
/// because `redox-fuzz`'s never-invalid-Rust invariant wants an answer here
/// rather than a panic.
pub(crate) fn control_text(expr: &Expr, indent: usize) -> String {
  let mut inner = Sink::new("");
  emit_control(&mut inner, expr, indent, false);
  let (text, _) = inner.finish("");
  text.trim_start().trim_end().to_string()
}
