pub(crate) mod clause;
pub(crate) mod fn_decl;
pub(crate) mod param;
pub(crate) mod pattern;
pub(crate) mod signature;

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
