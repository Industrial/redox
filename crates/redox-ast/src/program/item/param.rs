use crate::{Ident, TypeExpr};
use redox_lexer::Span;

/// One parameter in a signature.
///
/// The type is optional at the AST level because a `self` parameter has none,
/// and because rustc — not Redox — decides when one is required (doc 02 §6.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
  pub name: Ident,
  pub ty: Option<TypeExpr>,
  pub is_mut: bool,
  pub span: Span,
}
