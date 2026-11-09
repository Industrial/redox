use crate::emit::expr::emit_expr;
use crate::emit::single_line_if_else_max_width::SINGLE_LINE_IF_ELSE_MAX_WIDTH;
use crate::emit::sole_expression::sole_expression;
use crate::emit::unparen::unparen;
use redox_ast::Guard;

/// A two-armed guard ladder as one line, when rustfmt would write it that
/// way.
///
/// ```text
/// halve n                          if n % 2 == 0 { Some(n / 2) } else { None }
///   | n % 2 == 0 = Some (n / 2)
///   | otherwise  = None
/// ```
///
/// `None` unless **every** condition holds: exactly two rows, the second
/// `otherwise`, each body a single expression, and the result no wider than
/// `SINGLE_LINE_IF_ELSE_MAX_WIDTH`. Anything else is the multi-line form,
/// which is what rustfmt leaves alone in every other case.
///
/// `indent` is not part of the width test, because rustfmt's is not: the rule
/// measures the expression, as `chain_width` and `fn_call_width` do. It is
/// taken here only so the bodies are rendered at the right column should one
/// of them span lines — in which case the one-line form is refused anyway.
pub(crate) fn two_armed(guards: &[Guard], indent: usize) -> Option<String> {
  let [first, second] = guards else {
    return None;
  };
  let condition = first.condition.as_ref()?;
  if second.condition.is_some() {
    return None;
  }

  let then = sole_expression(first, indent)?;
  let otherwise = sole_expression(second, indent)?;
  let text = format!(
    "if {} {{ {then} }} else {{ {otherwise} }}",
    emit_expr(unparen(condition), indent)
  );
  if text.len() > SINGLE_LINE_IF_ELSE_MAX_WIDTH || text.contains('\n') {
    return None;
  }
  Some(text)
}
