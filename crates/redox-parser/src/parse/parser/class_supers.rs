use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::Ident;
use redox_lexer::TokenKind;

/// The `Show =>` or `(Show, Debug) =>` clause of a `class` head, if one is
/// here — doc 18 §2.5.
///
/// ```text
/// class Show => Shape          trait Shape: Show
/// class (Show, Debug) => Shape trait Shape: Show + Debug
/// class Shape                  trait Shape
/// ```
///
/// Trait names alone, where a signature's `constraints` reads a trait *and*
/// the variable it bounds. A superclass constrains `Self`, which has no
/// spelling to read, and plan 67 §6 keeps a bound on a trait parameter out of
/// scope — so there is nothing here for a second name to be.
///
/// Speculative for `constraints`' reason, and one token cheaper than it was:
/// `class Debug t => Shape t` and `class Shape t` began identically, so the
/// clause had to be read and abandoned. `class Show => Shape` and
/// `class Shape` differ at the `=>`, which is still past the first name — so
/// the rewind stays, and it is now a rewind over exactly one name list.
pub(crate) fn class_supers(p: &mut Parser) -> Result<Vec<Ident>, ParseError> {
  let mark = p.position();
  let mut found = Vec::new();

  // Haskell parenthesises a list of two or more; one needs no parentheses.
  let parenthesised = p.check(&TokenKind::LParen);
  if parenthesised {
    p.advance();
  }

  while let Some(name) = p.try_ident() {
    found.push(name);
    if !p.check(&TokenKind::Comma) {
      break;
    }
    p.advance();
  }

  if parenthesised && p.check(&TokenKind::RParen) {
    p.advance();
  }

  if found.is_empty() || !p.check(&TokenKind::FatArrow) {
    p.rewind(mark);
    return Ok(Vec::new());
  }
  p.advance();
  Ok(found)
}
