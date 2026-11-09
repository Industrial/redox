pub(crate) mod clause;
pub(crate) mod clause_body;
pub(crate) mod constructor;
pub(crate) mod constructor_fields;
pub(crate) mod data_decl;
pub(crate) mod field_decl;
pub(crate) mod fn_decl;
pub(crate) mod guard;
pub(crate) mod param;
pub(crate) mod pattern;
pub(crate) mod signature;
pub(crate) mod type_alias;

use crate::FnDecl;
use crate::program::item::data_decl::DataDecl;
use crate::program::item::type_alias::TypeAlias;
use redox_lexer::Span;

/// A top-level declaration.
///
/// Doc 18 §2.4 collapsed four of doc 03's declaration keywords into `data`
/// and `type`, which is why there are three variants here rather than six:
/// `struct`, `enum`, `union` and the tuple-struct spelling are all `data`.
/// `class` and `instance` (§2.5) are still to come.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
  Fn(FnDecl),
  Data(DataDecl),
  Type(TypeAlias),
}

impl Item {
  pub fn span(&self) -> Span {
    match self {
      Item::Fn(f) => f.span,
      Item::Data(d) => d.span,
      Item::Type(a) => a.span,
    }
  }
}
