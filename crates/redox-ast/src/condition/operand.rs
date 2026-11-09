/// One operand of a [`Condition`](crate::Condition) — plan 22.
///
/// Either an ordinary boolean expression, or doc 18 §3's pattern test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionOperand {
  /// `n > 0`, emitted verbatim.
  Expr(crate::Expr),
  /// `Some v <- x`, emitted as `let Some(v) = x`.
  ///
  /// The pattern is on the **left**, which is the order every other `<-`
  /// position already uses and the order Haskell reads — doc 18 §3.1 calls it
  /// *"`is` with the operands in Haskell's order"*. Rust's `let` puts it on
  /// the left too, so the lowering reorders nothing.
  Pattern {
    pattern: crate::Pattern,
    source: crate::Expr,
  },
}
