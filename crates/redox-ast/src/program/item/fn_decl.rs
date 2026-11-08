use crate::{Block, Ident, Signature};
use redox_lexer::{Span, Trivia};

/// `name = (params): Ret ->` followed by an indented block.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) added the signature; plan 01
/// shipped the zero-parameter, no-return-type form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
  pub name: Ident,
  pub sig: Signature,
  pub body: Block,
  pub span: Span,
  pub leading: Vec<Trivia>,
  pub trailing: Option<Trivia>,
}
