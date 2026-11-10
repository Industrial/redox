pub(crate) mod binding;

use crate::Expr;
use redox_lexer::{Span, Trivia};

/// One element of a [`Block`](crate::Block): an expression, and the trivia
/// that surrounds it.
///
/// # Why this survives plan 55 leaf 1, when `StmtKind` did not
///
/// Leaf 1 collapsed the statement/expression fork: there is one node kind,
/// and every body position holds an [`Expr`]. That could have made a block a
/// bare `Vec<Expr>` — and it does not, because **`Expr` carries no trivia**.
///
/// A comment attaches to a *line*, not to a subexpression. AGENTS.md makes
/// comment preservation load-bearing (doc 02 §10.1 corrects doc 01 §4.1), so
/// the trivia has to live somewhere, and the two candidates are this wrapper
/// or two more fields on every one of `Expr`'s forty-six forms. So `Stmt`
/// stays as the trivia carrier and keeps its name: it is still a block
/// element, and the thing that changed is that its payload is no longer a
/// kind of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stmt {
  pub expr: Expr,
  pub span: Span,
  pub leading: Vec<Trivia>,
  pub trailing: Option<Trivia>,
}
