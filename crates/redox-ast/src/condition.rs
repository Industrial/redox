pub(crate) mod operand;

/// An `if`, `while` or `case`-arm-guard condition — plan 22.
///
/// Doc 18 §3's third and fourth positions for `<-`: *"in a condition or
/// guard — pattern test, binding"*, replacing doc 03 §5.5's `is` with the
/// operands in Haskell's order.
///
/// ```text
/// if n > 0                     if n > 0 {
/// if Some v <- x               if let Some(v) = x {
/// if Some v <- x && v > 2      if let Some(v) = x && v > 2 {
/// ```
///
/// **A list of operands joined by `&&`**, because that is exactly what a Rust
/// let-chain is. The ordinary case is one operand and emits the expression it
/// always emitted — plan 22's criterion 8 asserts that, so the node does not
/// change the path plan 20 shipped.
///
/// Why a list and not a nest: `if let Some(v) = x && v > 2` is **one**
/// condition with two operands, and lowering it as two nested `if`s behaves
/// identically until an `else` is attached and differently from then on. Doc
/// 02 §5.2's failure mode with a delay fuse, so the shape is kept flat to
/// match Rust's own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Condition {
  /// One or more operands, joined by `&&`. Never empty.
  pub operands: Vec<crate::ConditionOperand>,
}

impl Condition {
  /// The ordinary, pattern-free condition — one expression operand.
  pub fn plain(expr: crate::Expr) -> Self {
    Self {
      operands: vec![crate::ConditionOperand::Expr(expr)],
    }
  }

  /// Whether any operand is a pattern test.
  ///
  /// Asked by one caller only: `until`, which lowers to `while !(c)` and so
  /// cannot hold one — `while !(let Some(v) = x)` is not Rust, and there is
  /// no honest alternative to reporting it. Plan 22 §1.3.
  pub fn has_pattern(&self) -> bool {
    self
      .operands
      .iter()
      .any(|o| matches!(o, crate::ConditionOperand::Pattern { .. }))
  }
}
