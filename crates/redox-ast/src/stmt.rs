use crate::StmtKind;
use redox_lexer::{Span, Trivia};

/// A statement, with the trivia that surrounds it.
///
/// Trivia lives on statements as well as items because an inline comment
/// attaches to a statement — without it, the lexer's "comments are never
/// discarded" property would hold only partly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stmt {
  pub kind: StmtKind,
  pub span: Span,
  pub leading: Vec<Trivia>,
  pub trailing: Option<Trivia>,
}
