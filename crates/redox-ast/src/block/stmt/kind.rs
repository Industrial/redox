use crate::Expr;

/// What a [`crate::Stmt`] is.
///
/// Plan 01 has only the expression form; `let` bindings are plan 03 and
/// control flow is plan 20.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StmtKind {
  Expr(Expr),
}
