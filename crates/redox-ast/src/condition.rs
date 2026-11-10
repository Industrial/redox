/// One operand of a condition.
pub(crate) mod operand;

use crate::{ConditionOperand, Expr};

/// An `if`, `while` or `case`-arm-guard condition — plan 22.
///
/// Doc 30 §3.1's pattern test: **Rust's `if let` with `let` dropped.**
///
/// ```text
/// if n > 0                    if n > 0 {
/// if Some v = x               if let Some(v) = x {
/// if Some v = x && v > 2      if let Some(v) = x && v > 2 {
/// ```
///
/// Spelled `<-` when doc 18 §3.1 gave the draw arrow four jobs, and `=` since
/// doc 30 made `<-` mean `await` and nothing else. The pattern stayed on the
/// left through both, so this node and every lowering below it never moved —
/// only the token the parser looks for did.
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
  ///
  /// An **associated function**: no `self` appears in the type, so plan 67's
  /// marked receiver reads it as one and emits `fn plain(expr: Expr) -> Self`.
  pub fn plain(expr: Expr) -> Self {
    Condition {
      operands: {
        let __rdx_t: Vec<ConditionOperand> = Vec::from([ConditionOperand::Expr(expr)]);
        __rdx_t
      },
    }
  }

  /// Whether any operand is a pattern test.
  ///
  /// Asked by one caller only: `until`, which lowers to `while !(c)` and so
  /// cannot hold one — `while !(let Some(v) = x)` is not Rust, and there is
  /// no honest alternative to reporting it. Plan 22 §1.3.
  ///
  /// A `case` ladder where the Rust wrote `matches!`, for the reason
  /// `primitive_type` records: `matches!`'s second argument is a pattern and
  /// plan 64 §6 discloses that a value-passing shape cannot carry one.
  pub fn has_pattern(&self) -> bool {
    self.operands.iter().any(|o| match o {
      ConditionOperand::Pattern { .. } => true,
      _ => false,
    })
  }
}
