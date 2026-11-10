use crate::program::item::visibility::Visibility;
use crate::{Ident, TypeExpr};
use redox_lexer::{Span, Trivia};

/// One `name : Type` field of a record constructor — doc 18 §2.4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDecl {
  pub name: Ident,
  pub ty: TypeExpr,
  /// Doc 03 §9.6's visibility. Private when the author wrote no marker,
  /// which is Rust's default and has no spelling.
  pub visibility: Visibility,
  /// The field's own comments and doc comments.
  ///
  /// Added by plan 48 leaf 2, which found them **silently discarded**: a
  /// `--- doc` on a field parsed and never reached the `.xrs`. AGENTS.md
  /// calls that rule load-bearing — *"comments are trivia, never discarded;
  /// they must reach the generated `.xrs`"* (doc 02 §10.1) — and no corpus
  /// file had a field doc comment, so nothing caught it.
  ///
  /// Found by porting `clause.rs`, which has four of them, for plan 48's
  /// conformance test. A count over the compiler's source cannot see this
  /// class of defect at all; compiling a real module can.
  pub leading: Vec<Trivia>,
  pub span: Span,
}
