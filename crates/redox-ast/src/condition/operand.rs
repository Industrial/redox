/// One operand of a [`Condition`](crate::Condition) — plan 22.
///
/// Either an ordinary boolean expression, or doc 18 §3's pattern test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionOperand {
  /// `n > 0`, emitted verbatim.
  Expr(crate::Expr),
  /// `Some v = x`, emitted as `let Some(v) = x`.
  ///
  /// The pattern is on the **left**, which is where Rust's `let` puts it — so
  /// the lowering reorders nothing and never has. Doc 18 §3.1 spelled this
  /// `<-`; doc 30 §3.1 spells it `=`, which is Rust's own form with the `let`
  /// dropped and costs no keyword.
  Pattern {
    pattern: crate::Pattern,
    source: crate::Expr,
  },
}
#[allow(unused_imports)]
use ConditionOperand::*;
