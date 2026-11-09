use crate::expr_text::expr_text;
use crate::pattern_text::pattern_text;
use crate::stmt_text::condition_text::condition_text;
use crate::stmt_text::indent_text::indent_text;
use redox_ast::CaseArm;

/// One `case` arm — `pattern -> body`, with an optional guard.
///
/// §3 row 7: **one space after the `->`**, never padding to align several
/// arms' bodies. The plan's reason is that alignment breaks on the next edit;
/// the stronger one is that it makes one arm's change show up as a diff in
/// every other arm.
///
/// The guard is a full condition, so `n | n > 65 -> "senior"` and
/// `y | Some (Some v) = y -> v` are one form with different operands.
pub(crate) fn case_arm_text(arm: &CaseArm, indent: usize, source: &str) -> String {
  let guard = match &arm.guard {
    Some(condition) => format!(" | {}", condition_text(condition, indent, source)),
    None => String::new(),
  };
  format!(
    "{}{}{} -> {}",
    indent_text(indent),
    pattern_text(&arm.pattern),
    guard,
    expr_text(&arm.body, 0, indent, source)
  )
}
