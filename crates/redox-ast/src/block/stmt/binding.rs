use crate::{Expr, Ident, TypeExpr};
use redox_lexer::Span;

/// `name = value`, `mut name = value`, or `name: Type = value`.
///
/// Immutable by default (doc 01 §4.3), so `mut` is the opt-in and lowers to
/// `let mut`. The type is optional because Redox leaves inference to rustc —
/// doc 02 §6.2's rule that every check rustc can make, rustc makes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
  pub name: Ident,
  pub ty: Option<TypeExpr>,
  pub value: Expr,
  pub is_mut: bool,
  pub span: Span,
}
