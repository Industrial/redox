use redox_ast::{Block, StmtKind};

/// The blocks a statement owns that belong to the **same** closure.
///
/// A lambda's body and a `try` block's are closures of their own, and neither
/// is a statement's block anyway — both are expressions, so neither can be
/// reached from here. That is why this function needs no exclusion list: the
/// statement kinds that own blocks are exactly the ones that do not open a
/// closure.
pub(crate) fn blocks_in_the_same_closure(kind: &StmtKind) -> Vec<&Block> {
  match kind {
    StmtKind::If {
      then_block,
      otherwise,
      ..
    } => match otherwise {
      Some(other) => vec![then_block, other],
      None => vec![then_block],
    },
    StmtKind::While { body, .. } | StmtKind::For { body, .. } | StmtKind::Loop { body, .. } => {
      vec![body]
    }
    StmtKind::Guard { otherwise, .. } => vec![otherwise],
    _ => Vec::new(),
  }
}
