use crate::{Expr, Pattern};
use redox_lexer::Span;

/// One arm of doc 18 §2.3's `case … of`.
///
/// ```text
/// label age = case age of        match age {
///   n | n > 65      -> "senior"      n if n > 65 => "senior",
///   18..=64         -> "adult"       18..=64 => "adult",
///   Teen or PreTeen -> "young"       Teen | PreTeen => "young",
///   _               -> "newborn"     _ => "newborn",
///                                }
/// ```
///
/// The arm arrow is `->`, not `=>`. That is the whole reason `=>` is free to
/// take up its Haskell job in constraints (doc 18 §7), and it is why doc 03's
/// `switch` keyword leaves with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseArm {
  pub pattern: Pattern,
  /// `n | n > 65` — the guard, lowered to Rust's own `if` arm guard.
  ///
  /// Doc 18 §5.4: **the first `|` after a `case` pattern opens the guard**,
  /// which is unambiguous only because pattern alternation keeps doc 03's
  /// `or` keyword rather than Rust's `|`. Unlike an equation's guard ladder
  /// this is one condition, not a list, because Rust has the construct
  /// already — the `match` arm guard — and doc 18 §1 holds the lowerings
  /// fixed wherever Rust already has the shape.
  pub guard: Option<crate::Condition>,
  /// The arm's value, after the `->`.
  ///
  /// An **expression**, and since plan 55 leaf 3 an indented block is one:
  /// a block body is `Expr::Do` with no label, which §10.4 records as
  /// Redox's only spelling of a block expression. So `-> do` and the
  /// marker-free block are the same node, which is what makes the marker
  /// deprecated rather than special-cased — and leaf 7 removes a keyword,
  /// not a form.
  pub body: Expr,
  pub span: Span,
}
