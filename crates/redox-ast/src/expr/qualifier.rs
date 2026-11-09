use crate::{Expr, Ident};

/// One qualifier to the right of a comprehension's bar — doc 18 §3.2.
///
/// ```text
/// [ x * y | x <- 1..3, y <- 1..3, x != y ]
///           ^^^^^^^^^  ^^^^^^^^^  ^^^^^^
///           generator  generator  guard
/// ```
///
/// The bar, the draw arrow and the comma do what doc 03's `for` and `when`
/// clauses did. Doc 03's *second* comprehension level — Dart's element-wise
/// `if`/`for` inside a literal, with its `push`/`insert` lowering — is
/// deleted: Haskell has one comprehension and so does this.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Qualifier {
  /// `x <- xs` — a generator. `<-` in its fourth position (doc 18 §3), and
  /// it reads the way it reads everywhere else: a name drawn out of
  /// something.
  Draw { name: Ident, source: Expr },
  /// `x != y` — a guard, filtering what the generators produced.
  Guard(Expr),
}
