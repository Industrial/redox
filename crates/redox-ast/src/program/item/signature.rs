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
  /// `name :: const T -> U` — doc 18 §2.2's `const fn`.
  ///
  /// A marker between `::` and the spine, consumed before the types are
  /// captured. It has to be: a type in Redox is verbatim text (doc 03
  /// §14.1), and leaving `const` in one would make the first parameter's
  /// type `const u32`, which is not a Rust type — which is exactly what
  /// happened before plan 16.
  pub is_const: bool,
  /// `name :: async T -> U` — doc 03 §14.3's `async fn`, plan 39.
  ///
  /// The same shape as [`is_const`](Self::is_const) and for the same reason:
  /// a type in Redox is verbatim text, so leaving `async` in one would make
  /// the first parameter's type `async u64`, which is not a Rust type. Plan
  /// 16 found that with `const` and plan 39 inherits the fix.
  ///
  /// Rust has no `const async fn`, and that is **rustc's** to say: writing
  /// both emits both and rustc objects, which is AGENTS.md's rule.
  pub is_async: bool,
  pub span: Span,
}
