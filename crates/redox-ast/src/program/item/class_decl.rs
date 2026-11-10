use crate::Ident;

use crate::program::item::class_member::ClassMember;

use crate::program::item::visibility::Visibility;

use redox_lexer::{Span, Trivia};

/// A `class` declaration — doc 18 §2.5's `trait`.
///
/// ```text
/// class Debug => Shape              trait Shape: Debug {
///   type Unit                           type Unit;
///   const SIDES : u32                  const SIDES: u32;
///   area : &self -> f64          →     fn area(&self) -> f64;
///   describe : &self -> String         fn describe(&self) -> String {
///   describe self = "…"                     "…"
///                                     }
///                                   }
/// ```
///
/// Rust's trait system is Haskell's class system with different words, so doc
/// 18 §2.5 changes the words back: `class` stops meaning doc 01 §17.7's
/// "struct plus impl" and means `trait`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassDecl {
  pub name: Ident,
  /// The trait's **own** parameters — the `t` in `class Convert t`, emitted
  /// as `trait Convert<T>`.
  ///
  /// Empty for `class Shape`, which is the common case. Plan 67 is what made
  /// this a list: the implementing type used to occupy the one slot there
  /// was, so a second lowercase name became a *method* generic and
  /// `impl From<u32>` and `impl From<f64>` for one type could not both be
  /// written. With the receiver marked (`&self` in a type) the slot is free,
  /// and the names in it are the trait's.
  pub params: Vec<Ident>,
  /// `Debug =>` — the superclasses, which become Rust's supertrait bounds.
  ///
  /// Trait names alone, with no variable: a superclass constrains `Self`, and
  /// a bound on a trait *parameter* is plan 67 §6's out-of-scope row.
  pub supers: Vec<Ident>,
  pub members: Vec<ClassMember>,
  /// Doc 03 §9.6's visibility. Private when the author wrote no marker,
  /// which is Rust's default and has no spelling.
  pub visibility: Visibility,
  pub span: Span,
  pub leading: Vec<Trivia>,
}
