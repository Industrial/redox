use crate::Ident;
use crate::program::item::constructor_fields::ConstructorFields;
use redox_lexer::Span;

/// One constructor of a `data` declaration — doc 18 §2.4.
///
/// `Click { x : i32, y : i32 }`, `Key char`, `Close`. Which Rust form it
/// becomes depends on the *declaration*, not on the constructor: the same
/// `Marker` is `struct Marker;` alone and `Marker,` inside an enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constructor {
  pub name: Ident,
  pub fields: ConstructorFields,
  pub span: Span,
}
