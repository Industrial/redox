use crate::program::item::visibility::Visibility;
use crate::{Ident, TypeExpr};
use redox_lexer::Span;

/// One `name :: Type` field of a record constructor — doc 18 §2.4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDecl {
  pub name: Ident,
  pub ty: TypeExpr,
  /// Doc 03 §9.6's visibility. Private when the author wrote no marker,
  /// which is Rust's default and has no spelling.
  pub visibility: Visibility,
  pub span: Span,
}
