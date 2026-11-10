use crate::{Param, TypeExpr};
use redox_lexer::Span;

/// A function's parameters and return type.
///
/// A zero-parameter function writes no list at all (`main : ()` / `main =`),
/// which is the form plan 01 shipped.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Signature {
  pub params: Vec<Param>,
  /// The declared return type, **verbatim and including `()`**.
  ///
  /// `None` is unreachable from the parser — doc 18 §2.2 makes the signature
  /// line mandatory and `parse_spine` always yields a last type — so it means
  /// only *this was built by [`Default`]*. [`returns_unit`](Self::returns_unit)
  /// reads it as `()` accordingly, which is also what Rust means by a missing
  /// return type.
  ///
  /// **`main : ()` arrives here as `Some("()")`**, and plan 55 leaf 4 is why.
  /// The parser used to drop a unit return on the floor —
  /// `(ret.text != "()").then(…)` — because the emitter must not write
  /// `-> ()`; rustfmt removes it and plan 04's rustfmt-clean guarantee would
  /// have to defend the noise. But that made a declared `()` indistinguishable
  /// from no declaration at all, and §1.6's rule — *a block whose value would
  /// be unit must say `()`* — is decidable only from the declared type. So the
  /// omission moved to where it belongs, `emit::signature`, and the tree
  /// records what the author wrote. That is the same correction
  /// `redox-fmt`'s `signature_line` asks for in the general case: the AST keeps
  /// the source spelling and the emitter applies Rust's convention.
  pub ret: Option<TypeExpr>,
  /// The generic parameters, **already in Rust's spelling** and in Rust's
  /// required order: lifetimes first, then type variables, each in order of
  /// first appearance.
  ///
  /// Doc 18 §§2.2 and 5.3: "lowercase type names are type variables … the
  /// generic list is the order of their first appearance, lifetimes
  /// included". Synthesised rather than written, which is the whole point —
  /// `largest : Ord t => &[t] -> &t` declares `t` nowhere.
  pub generics: Vec<String>,
  /// The `=>` clause's bounds, as `(parameter, trait)` pairs in source order.
  ///
  /// Pairs rather than a map, because the emitter's output must be
  /// byte-stable and a map's iteration order is not (see `redox-emit`'s own
  /// module doc).
  pub bounds: Vec<(String, String)>,
  /// `name : const T -> U` — doc 18 §2.2's `const fn`.
  ///
  /// A marker between `::` and the spine, consumed before the types are
  /// captured. It has to be: a type in Redox is verbatim text (doc 03
  /// §14.1), and leaving `const` in one would make the first parameter's
  /// type `const u32`, which is not a Rust type — which is exactly what
  /// happened before plan 16.
  pub is_const: bool,
  /// `name : async T -> U` — doc 03 §14.3's `async fn`, plan 39.
  ///
  /// The same shape as [`is_const`](Self::is_const) and for the same reason:
  /// a type in Redox is verbatim text, so leaving `async` in one would make
  /// the first parameter's type `async u64`, which is not a Rust type. Plan
  /// 16 found that with `const` and plan 39 inherits the fix.
  ///
  /// Rust has no `const async fn`, and that is **rustc's** to say: writing
  /// both emits both and rustc objects, which is AGENTS.md's rule.
  pub is_async: bool,
  /// `name : unsafe T -> U` — Rust's `unsafe fn`, plan 45 leaf 2.
  ///
  /// Read in the same place and for the same reason as
  /// [`is_async`](Self::is_async): left in a type, `unsafe` would make the
  /// first parameter's type `unsafe u32`.
  ///
  /// Doc 01 §26 names Unsafe Rust as something that *"must remain
  /// possible"*, and doc 03 §23 item 2 rules out reaching it through the
  /// escape hatch — so it is spelled rather than deferred.
  pub is_unsafe: bool,
  pub span: Span,
}

impl Signature {
  /// Whether this function's value is `()` — a declared `()` return type, or
  /// no declaration at all.
  ///
  /// The one question two unrelated readers of [`ret`](Self::ret) both ask:
  /// `emit::signature` to decide whether to write `-> T`, and
  /// `emit_clauses` to decide whether the body's last element is a tail. Both
  /// used to spell it `ret.is_some()`, which was the same question only for as
  /// long as a unit return was discarded at parse time (plan 55 leaf 4).
  pub fn returns_unit(&self) -> bool {
    self.ret.as_ref().is_none_or(|ret| ret.text == "()")
  }
}
