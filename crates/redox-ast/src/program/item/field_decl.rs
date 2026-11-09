use crate::{Ident, TypeExpr};
use redox_lexer::Span;

/// One `name :: Type` field of a record constructor — doc 18 §2.4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDecl {
  pub name: Ident,
  pub ty: TypeExpr,
  pub span: Span,
}
