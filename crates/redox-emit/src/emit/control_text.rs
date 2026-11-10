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
/// Reached by the indented `if`, `while`, `for` and `loop` from an expression
/// position. Leaf 1 wrote it for a position the grammar could not yet produce
/// — `prefix` had no production for any of them — rather than leaving a
/// `todo!()`, because an AST node with no lowering is a delay fuse and because
/// `redox-fuzz`'s never-invalid-Rust invariant wants an answer here rather
/// than a panic. **Leaf 2 made that position real**, for the indented `if`.
///
/// # `tail` is `true`, and it was `false` for one leaf
///
/// An expression's value *is* its value, so the branches of an `if` reached
/// through here are tail positions by construction. While the position was
/// unreachable the choice was unobservable, and leaf 1 passed `false`; leaf 2
/// then produced
///
/// ```text
/// label = if n > 9
///   "big"
/// else
///   "small"
/// ```
///
/// which emitted `"big";` and earned rustc's `E0308` — the same defect leaf 0
/// fixed for tail position, in the one path leaf 0 could not reach.
///
/// `true` is correct for the other three forms as well, because it does not
/// reach them: `emit_control` forwards `tail` only to an `if`'s branches, and
/// only when there is an `else`. A `while`, a `for`, a `loop` and an
/// `else`-less `if` all ignore it, which is right — every one of them is `()`
/// in Rust.
pub(crate) fn control_text(expr: &Expr, indent: usize) -> String {
  let mut inner = Sink::new("");
  emit_control(&mut inner, expr, indent, true);
  let (text, _) = inner.finish("");
  text.trim_start().trim_end().to_string()
}
