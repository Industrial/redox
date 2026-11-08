pub(crate) mod fn_decl;

use crate::FnDecl;
use redox_lexer::Span;

/// A top-level declaration.
///
/// Plan 01's slice has exactly one form; structs, enums, traits and impls are
/// plans 24 and 25.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
  Fn(FnDecl),
}

impl Item {
  pub fn span(&self) -> Span {
    match self {
      Item::Fn(f) => f.span,
    }
  }
}
