use crate::emit::expr::emit_expr;
use crate::emit::unparen::unparen;
use redox_ast::{Condition, ConditionOperand};

/// The Rust text of an `if`, `while` or arm-guard condition — plan 22.
///
/// ```text
/// n > 0                     n > 0
/// Some v <- x           →   let Some(v) = x
/// Some v <- x && v > 2      let Some(v) = x && v > 2
/// ```
///
/// Operands are joined by `&&` and the result is **flat**, because a Rust
/// let-chain is flat. Nesting one `if` per operand behaves identically until
/// an `else` is attached and differently from then on — doc 02 §5.2's failure
/// mode with a delay fuse — so the shape is kept as Rust's.
///
/// The one-operand expression case emits exactly what plan 20 emitted, which
/// plan 22's criterion 8 asserts: a condition node that changed the old path
/// would be a regression dressed as a feature.
pub(crate) fn emit_condition(condition: &Condition, indent: usize) -> String {
  condition
    .operands
    .iter()
    .map(|operand| match operand {
      ConditionOperand::Expr(e) => emit_expr(unparen(e), indent),
      // `let P = e`, with the pattern on the left in both languages — doc 18
      // §3.1's "operands in Haskell's order" happens to be Rust's order too,
      // so nothing is reordered.
      ConditionOperand::Pattern { pattern, source } => {
        format!(
          "let {} = {}",
          pattern.rust_text(),
          emit_expr(unparen(source), indent)
        )
      }
    })
    .collect::<Vec<_>>()
    .join(" && ")
}
