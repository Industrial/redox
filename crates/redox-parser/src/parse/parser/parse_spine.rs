use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::capture_type::capture_type;
use crate::parse::parser::type_stop::TypeStop;
use redox_ast::TypeExpr;
use redox_lexer::TokenKind;

/// Parse a signature line's type spine: the types between top-level `->`s.
///
/// Doc 18 §2.2. `load :: &str -> Result<Config, Error>` has the spine
/// `[&str, Result<Config, Error>]` — the last element is the return type and
/// the rest are parameter types, matched against the equation's binders.
///
/// Every atom between the arrows is still a **verbatim Rust type** (doc 03
/// §14.1), so `capture_type` does the work unchanged. It already stops at a
/// top-level `->` and already keeps going through `Fn(i32) -> i32`, which is
/// the refinement plan 03 added for exactly this shape — so the spine costs
/// nothing but the loop.
///
/// Only the spine becomes Haskell; doc 18 §8 declines type application by
/// juxtaposition *in types* precisely so that this stays true.
pub(crate) fn parse_spine(p: &mut Parser) -> Result<Vec<TypeExpr>, ParseError> {
  let mut spine = vec![capture_type(
    p,
    "expected a type after `::`",
    TypeStop::Greedy,
  )?];
  while p.check(&TokenKind::Arrow) {
    p.advance();
    spine.push(capture_type(
      p,
      "expected a type after `->`",
      TypeStop::Greedy,
    )?);
  }
  Ok(spine)
}
