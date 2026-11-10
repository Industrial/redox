use crate::expr_text::expr_text;
use crate::pattern_text::pattern_text;
use redox_ast::{Condition, ConditionOperand};

/// A condition — one or more operands joined by `&&`, each either an
/// expression or a pattern test.
///
/// A pattern test is `pattern = source`, which doc 30 settled: `<-` means
/// `await` and nothing else, so the condition form took `=`. §3 notes that a
/// construct with one spelling needs no canonical form, and this is one of
/// them.
///
/// # The required power depends on how many operands there are
///
/// With **one** operand the condition simply *is* that expression, so it is
/// printed at power 0 — otherwise `if a || b` would come out as `if (a || b)`,
/// since `||` binds looser than the `&&` that joins operands.
///
/// With **more than one** the operands are joined by `&&`, so each must bind
/// tighter than `&&` or be parenthesised. `10` is `&&`'s power plus one: the
/// tighter of its two sides, which is the safe choice for a flattened list
/// where no operand is singled out as the rightmost.
pub(crate) fn condition_text(condition: &Condition, indent: usize, source: &str) -> String {
  let required = if condition.operands.len() == 1 { 0 } else { 10 };
  condition
    .operands
    .iter()
    .map(|operand| match operand {
      ConditionOperand::Expr(expr) => expr_text(expr, required, indent, source),
      ConditionOperand::Pattern {
        pattern,
        source: from,
      } => format!(
        // Plan 60: scrutinee first, joined by `is`. The node kept its field
        // order — `pattern` then `source` — so only the printing reversed,
        // which is the same shape the parser's change took.
        "{} is {}",
        expr_text(from, required, indent, source),
        pattern_text(pattern)
      ),
    })
    .collect::<Vec<_>>()
    .join(" && ")
}
