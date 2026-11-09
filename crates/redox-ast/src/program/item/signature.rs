use crate::{Param, TypeExpr};
use redox_lexer::Span;

/// A function's parameters and return type.
///
/// Both are optional: doc 01 §5.1 makes the return type optional, and a
/// zero-parameter function writes no list at all (`main :: ()` / `main =`), which is the
/// form plan 01 shipped.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Signature {
  pub params: Vec<Param>,
  pub ret: Option<TypeExpr>,
  /// The generic parameters, **already in Rust's spelling** and in Rust's
  /// required order: lifetimes first, then type variables, each in order of
  /// first appearance.
  ///
  /// Doc 18 §§2.2 and 5.3: "lowercase type names are type variables … the
  /// generic list is the order of their first appearance, lifetimes
  /// included". Synthesised rather than written, which is the whole point —
  /// `largest :: Ord t => &[t] -> &t` declares `t` nowhere.
  pub generics: Vec<String>,
  /// The `=>` clause's bounds, as `(parameter, trait)` pairs in source order.
  ///
  /// Pairs rather than a map, because the emitter's output must be
  /// byte-stable and a map's iteration order is not (see `redox-emit`'s own
  /// module doc).
  pub bounds: Vec<(String, String)>,
  pub span: Span,
}
