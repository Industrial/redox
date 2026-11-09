use crate::Ident;
use crate::program::item::constructor::Constructor;
use crate::program::item::visibility::Visibility;
use redox_lexer::{Span, Trivia};

/// A `data` or `newtype` declaration — doc 18 §2.4.
///
/// ```text
/// data Point = Point { x :: f64, y :: f64 }     #[derive(Debug, Clone)]
///   deriving (Debug, Clone)                     struct Point { x: f64, y: f64 }
///
/// data Event                                    enum Event {
///   = Click { x :: i32, y :: i32 }                  Click { x: i32, y: i32 },
///   | Key char                              →       Key(char),
///   | Close                                         Close,
///                                               }
/// newtype Meters = Meters f64                   struct Meters(f64);
/// data Marker = Marker                          struct Marker;
/// ```
///
/// **One constructor whose name matches the type is a `struct`; anything else
/// is an `enum`.** The rule is syntactic, which doc 18 §2.4 says is the whole
/// requirement — it is decidable from the source text alone, doc 03 §2's
/// admission test 1, and needs no name resolution.
///
/// Four Redox keywords collapse into two here: `struct`, `enum` and the
/// `union` and tuple-struct spellings are all `data`, and `derive` as a
/// standalone keyword becomes `deriving` as a clause of the declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDecl {
  pub name: Ident,
  /// One or more, in source order.
  pub constructors: Vec<Constructor>,
  /// The `deriving (…)` list, which becomes `#[derive(…)]`.
  pub derives: Vec<Ident>,
  /// Whether the author wrote `newtype`.
  ///
  /// Kept even though it changes no output: `newtype Meters = Meters f64` and
  /// `data Meters = Meters f64` lower identically, because Rust has one
  /// single-field tuple struct and not two. The flag is what lets the
  /// formatter re-spell the declaration the way it was written, and what
  /// would let a later plan reject `newtype` with more than one field — a
  /// check Haskell makes and Rust has no equivalent of.
  pub is_newtype: bool,
  /// `union Reg = Reg { a :: u32, b :: f32 }` — plan 45 leaf 4.
  ///
  /// Doc 18 §2.4's rule reads a shape off the constructors, and a union is
  /// not a shape — it is a struct's shape with overlapping storage, which no
  /// arrangement of constructors could select. So it takes a keyword, as
  /// `newtype` does for the same kind of reason.
  ///
  /// Rust requires named fields in a union and permits exactly one "variant";
  /// neither is checked here. A `union` with two constructors emits two
  /// variants and rustc rejects it, which is the every-check-rustc-makes
  /// corollary rather than an omission.
  pub is_union: bool,
  /// Doc 03 §9.6's visibility. Private when the author wrote no marker,
  /// which is Rust's default and has no spelling.
  pub visibility: Visibility,
  pub span: Span,
  pub leading: Vec<Trivia>,
}
