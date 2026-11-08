use crate::{Block, Ident};
use redox_lexer::{Span, Trivia};

/// `name = ->` followed by an indented block.
///
/// Parameters, type annotations and return types are plan 03; plan 01's
/// grammar has a bare name and a body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
  pub name: Ident,
  pub body: Block,
  pub span: Span,
  pub leading: Vec<Trivia>,
  pub trailing: Option<Trivia>,
}
