use crate::Ident;
use redox_lexer::Span;

/// One `Trait t` constraint, from a `=>` clause — doc 18 §2.5.
///
/// `class Debug t => Shape t` makes `Debug` a supertrait of `Shape`;
/// `instance Display t => Describe (Wrapper t)` makes it a bound on the
/// impl's generic parameter. The same shape serves both, because in Haskell
/// it *is* the same shape.
///
/// `=>` means this and nothing else. Doc 18 §7 retires it as an arm arrow
/// precisely so it can take up its Haskell job here, which is why `case` arms
/// use `->`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
  pub trait_name: Ident,
  /// The variable constrained — the `t` in `Debug t`.
  pub var: Ident,
  pub span: Span,
}
