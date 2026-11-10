use crate::{Expr, Pattern, TypeExpr};

use redox_lexer::Span;

/// `name = value`, `mut name = value`, `name: Type = value`, or doc 03 §6.5's
/// destructuring — `(a, b) = point`.
///
/// Immutable by default (doc 01 §4.3), so `mut` is the opt-in and lowers to
/// `let mut`. The type is optional because Redox leaves inference to rustc —
/// doc 02 §6.2's rule that every check rustc can make, rustc makes.
///
/// # The left side is a **pattern**, as of plan 33
///
/// `(a, b) = point` *is* a binding, so it is this node with a tuple pattern
/// rather than a second statement kind that would duplicate every binding
/// behaviour. A plain `x = 1` is a `Pattern::Binding`, which emits exactly
/// what it emitted before — plan 33's criterion 8 asserts that, because this
/// is the path every existing golden takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
  pub pattern: Pattern,
  pub ty: Option<TypeExpr>,
  pub value: Expr,
  pub is_mut: bool,
  pub span: Span,
}
