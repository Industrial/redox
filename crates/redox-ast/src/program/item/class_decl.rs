use crate::Ident;
use crate::program::item::class_member::ClassMember;
use crate::program::item::constraint::Constraint;
use crate::program::item::visibility::Visibility;
use redox_lexer::{Span, Trivia};

/// A `class` declaration — doc 18 §2.5's `trait`.
///
/// ```text
/// class Debug t => Shape t          trait Shape: Debug {
///   type Unit                           type Unit;
///   const SIDES : u32                  const SIDES: u32;
///   area : &t -> f64             →     fn area(&self) -> f64;
///   describe : &t -> String            fn describe(&self) -> String {
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
  /// The class variable — the `t` in `class Shape t`.
  ///
  /// Named rather than positional because it is what decides which parameter
  /// becomes `self`: doc 18 §2.5 says "the first parameter typed by the class
  /// variable lowers to `self`", which is the one place Haskell's
  /// dictionary-passing and Rust's receiver differ in spelling but not in
  /// content.
  pub var: Ident,
  /// `Debug t =>` — the superclasses, which become Rust's supertrait bounds.
  pub supers: Vec<Constraint>,
  pub members: Vec<ClassMember>,
  /// Doc 03 §9.6's visibility. Private when the author wrote no marker,
  /// which is Rust's default and has no spelling.
  pub visibility: Visibility,
  pub span: Span,
  pub leading: Vec<Trivia>,
}
