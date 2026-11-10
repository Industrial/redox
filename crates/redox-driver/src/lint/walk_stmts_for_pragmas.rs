use crate::lint::allow_pragma::allow_pragma;
use redox_ast::{Stmt, StmtKind};
use redox_diag::Span;

/// Walk a statement run for suppression pragmas — plan 53 leaf 4.
///
/// A pragma leading a statement suppresses that statement's **own span**,
/// which runs to the end of everything the statement owns — so an attribute
/// above an `if` covers its branches, and one above a `for` covers its body.
///
/// Then the blocks each statement owns, so a pragma inside one is found too.
/// The match is **exhaustive with no wildcard**, for the reason the emitter's
/// trivia match is: a new statement kind that owns a block has to be
/// considered here rather than quietly excluded.
///
/// # Where this stops, and why that is safe
///
/// Not into the blocks an **expression** owns — `do`, `async`, `unsafe`,
/// `try` and a lambda body. Reaching those needs an expression walk, which is
/// the thing
/// [`unwrap_call_sites`](super::unwrap_call_sites::unwrap_call_sites) refuses,
/// and the refusal is affordable here in a way it is not there: a statement
/// this walk does not reach keeps its warning, so the author sees `RDX1001`
/// beside the pragma they wrote and the gap reports itself. Detection going
/// wrong the same way would print nothing and look like a clean program.
///
/// The declaration-level pragma covers these cases in the meantime: an
/// attribute on the enclosing function suppresses its whole span, expression
/// blocks included.
pub(crate) fn walk_stmts_for_pragmas(stmts: &[Stmt], found: &mut Vec<Span>) {
  for stmt in stmts {
    if stmt.leading.iter().any(allow_pragma) {
      found.push(stmt.span);
    }

    match &stmt.kind {
      StmtKind::If {
        then_block,
        otherwise,
        ..
      } => {
        walk_stmts_for_pragmas(&then_block.stmts, found);
        if let Some(other) = otherwise {
          walk_stmts_for_pragmas(&other.stmts, found);
        }
      }
      StmtKind::While { body, .. } | StmtKind::For { body, .. } | StmtKind::Loop { body, .. } => {
        walk_stmts_for_pragmas(&body.stmts, found);
      }
      StmtKind::Guard { otherwise, .. } => walk_stmts_for_pragmas(&otherwise.stmts, found),
      StmtKind::Expr(_)
      | StmtKind::Binding(_)
      | StmtKind::Return(_)
      | StmtKind::Draw { .. }
      | StmtKind::Await { .. }
      | StmtKind::Assign { .. }
      | StmtKind::Break { .. }
      | StmtKind::Continue { .. }
      | StmtKind::Assert(_) => {}
    }
  }
}
