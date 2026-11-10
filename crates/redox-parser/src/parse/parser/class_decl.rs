use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::class_body::class_body;
use crate::parse::parser::constraints::constraints;
use crate::parse::parser::method_context::MethodContext;
use redox_ast::{ClassDecl, Visibility};
use redox_lexer::{Keyword, TokenKind, Trivia};

/// Parse doc 18 §2.5's `class` — Rust's `trait`.
///
/// ```text
/// class Debug t => Shape t     trait Shape: Debug {
///   type Unit                      type Unit;
///   const SIDES : u32         →   const SIDES: u32;
///   area : &t -> f64              fn area(&self) -> f64;
///                              }
/// ```
///
/// Rust's trait system is Haskell's class system with different words, so the
/// words change back: `class` stops meaning doc 01 §17.7's "struct plus impl"
/// and means `trait`.
///
/// The constraint clause is read speculatively, because `class Shape t` and
/// `class Debug t => Shape t` begin identically — see `constraints`.
pub(crate) fn class_decl(
  p: &mut Parser,
  leading: Vec<Trivia>,
  start: usize,
  visibility: Visibility,
) -> Result<ClassDecl, ParseError> {
  p.expect(&TokenKind::Kw(Keyword::Class), "expected `class`")?;
  let supers = constraints(p)?;
  let name = p.ident("expected the class's name")?;
  let var = p.ident("expected the class variable, as in `class Shape t`")?;

  let members = class_body(p, &MethodContext::for_class(&var.text))?;

  Ok(ClassDecl {
    name,
    var,
    supers,
    members,
    visibility,
    span: (start, p.prev_span().1),
    leading,
  })
}
