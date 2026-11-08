use crate::Stmt;
use redox_lexer::Span;

/// An indented sequence of statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
  pub stmts: Vec<Stmt>,
  pub span: Span,
}
