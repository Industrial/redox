use redox_ast::{Condition, ConditionOperand};

/// Whether rustfmt would break this condition across lines.
pub(super) fn breaks(condition: &Condition) -> bool {
  let n = condition.operands.len();
  let has_let = condition.has_pattern();
  let first_is_let = matches!(
    condition.operands.first(),
    Some(ConditionOperand::Pattern { .. })
  );
  has_let && (n > 2 || (n == 2 && first_is_let))
}
