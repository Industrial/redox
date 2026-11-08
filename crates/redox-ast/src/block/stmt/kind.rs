use crate::{Binding, Expr};

/// What a [`crate::Stmt`] is.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) added bindings and `return`;
/// control flow is plan 20.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StmtKind {
  Expr(Expr),
  Binding(Binding),
  /// `return` with an optional value.
  Return(Option<Expr>),
}
