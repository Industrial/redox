use crate::{Expr, Ident};

/// One qualifier to the right of a comprehension's bar — doc 18 §3.2.
///
/// ```text
/// [ x * y | x in 1..3, y in 1..3, x != y ]
///           ^^^^^^^^^  ^^^^^^^^^  ^^^^^^
///           generator  generator  guard
/// ```
///
/// The generator's word is `in` as of doc 30 §3.2, not doc 18 §3.2's own
/// `<-`: the token went to `await`. Doc 30 §4.2 records this as that
/// conversion's real aesthetic loss, since the comprehension was the one
/// position where doc 18's borrowing of `<-` was faithful to Haskell.
///
/// The bar, the generator word and the comma do what doc 03's `for` and
/// `when` clauses did. Doc 03's *second* comprehension level — Dart's element-wise
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
