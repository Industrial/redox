use crate::Ident;

use crate::program::item::constructor_fields::ConstructorFields;

use redox_lexer::{Span, Trivia};

/// One constructor of a `data` declaration — doc 18 §2.4.
///
/// `Click { x : i32, y : i32 }`, `Key char`, `Close`. Which Rust form it
/// becomes depends on the *declaration*, not on the constructor: the same
/// `Marker` is `struct Marker;` alone and `Marker,` inside an enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constructor {
  pub name: Ident,
  pub fields: ConstructorFields,
  /// The constructor's own comments and doc comments.
  ///
  /// Added by plan 68, which found them **silently discarded** — and found it
  /// the way plan 48 found the identical defect on [`crate::FieldDecl`] one
  /// node over: by porting real compiler source and reading the output.
  ///
  /// Plan 48 recorded that *"no corpus file had a field doc comment, so
  /// nothing caught it"*. The module it ported, `clause.rs`, is a **struct**
  /// — so it has no enum variants, and a constructor doc comment appeared in
  /// no corpus file either. `expr/unary_op.rs` has five, one per variant, and
  /// the port lost all five.
  ///
  /// That is the argument for porting a whole crate rather than one module,
  /// made concrete: the module plan 48 chose could not have found this.
  pub leading: Vec<Trivia>,
  pub span: Span,
}
