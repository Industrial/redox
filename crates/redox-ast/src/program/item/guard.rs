use crate::{Block, Expr};

use redox_lexer::Span;

/// One `| cond = body` row of doc 18 §2.3's guard ladder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guard {
  /// The condition, or `None` for `otherwise`.
  ///
  /// `otherwise` is Haskell's name for the final catch-all and lowers to a
  /// bare `else` — not to `if true`, which would leave rustc unable to see
  /// that the ladder is total, and would cost every guarded function with a
  /// return type a `return` its author did not write.
  pub condition: Option<Expr>,
  /// The row's value, as a block of one tail statement — so the emitter has
  /// one body shape to handle rather than two.
  pub body: Block,
  pub span: Span,
}
