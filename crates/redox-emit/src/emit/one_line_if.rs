use crate::emit::emit_condition::emit_condition;
use crate::emit::expr::emit_expr;
use crate::emit::single_line_if_else_max_width::SINGLE_LINE_IF_ELSE_MAX_WIDTH;
use crate::emit::unparen::unparen;
use redox_ast::{Block, Condition};

/// An `if … else …` on one line, when rustfmt would put it there.
///
/// rustfmt joins a two-armed if-else whose rendering fits
/// `single_line_if_else_max_width` — 50 by default, pinned in this crate — so
/// an emitter that always writes the multi-line form produces Rust that
/// `generated_rust_is_rustfmt_clean` rejects.
///
/// `two_armed` is the same decision for a **guard ladder**, and it came first.
/// This is the decision for an `if` reached as an **expression**, which plan 55
/// leaf 2 made possible and did not ask about: `examples/local.sdx`'s
/// `let shade = if lifted > LIMIT { Deep } else { Pale };` is 48 characters
/// and rustfmt collapsed it.
///
/// `None` whenever the one-line form does not apply — no `else`, a branch that
/// is not a single plain expression, a rendering that is too wide, or one that
/// already contains a newline. Every one of those falls through to the
/// multi-line form, which is always correct if sometimes wider than rustfmt
/// would choose.
///
/// The width is measured on the `if … else …` text alone rather than on the
/// finished line, which is the same approximation `two_armed` makes and for
/// the same reason: the caller's prefix is not known here, and rustfmt's own
/// rule is about the expression rather than the statement.
pub(crate) fn one_line_if(
  condition: &Condition,
  then_block: &Block,
  otherwise: Option<&Block>,
  indent: usize,
) -> Option<String> {
  let other = otherwise?;
  let then = then_block.sole_expression()?;
  let otherwise = other.sole_expression()?;
  if then.is_statement_form() || otherwise.is_statement_form() {
    return None;
  }

  let text = format!(
    "if {} {{ {} }} else {{ {} }}",
    emit_condition(condition, indent),
    emit_expr(unparen(then), indent),
    emit_expr(unparen(otherwise), indent)
  );
  if text.len() > SINGLE_LINE_IF_ELSE_MAX_WIDTH || text.contains('\n') {
    return None;
  }
  Some(text)
}
