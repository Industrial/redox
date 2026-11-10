use crate::Block;

use crate::program::item::guard::Guard;

/// A clause's right-hand side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClauseBody {
  /// `= body`.
  Plain(Block),
  /// `| cond = body` repeated, in order.
  ///
  /// Doc 18 §2.3 calls the guard bar *"the single highest-yield token in the
  /// redesign"*, because Haskell puts it in both equations and case arms and
  /// Redox had grown three constructs over the same ground: `cond`, `when`
  /// arm guards, and §6.2's relational patterns. All three are deleted by this
  /// one list, and the scrutinee temporary the relational patterns needed goes
  /// with them.
  Guarded(Vec<Guard>),
}
#[allow(unused_imports)]
use ClauseBody::*;

impl ClauseBody {
  /// The block a plain clause holds, if it is plain.
  pub fn plain(&self) -> Option<&Block> {
    match self {
      Plain(block) => Some(block),
      Guarded(_) => None,
    }
  }
}
