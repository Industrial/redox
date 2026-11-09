use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::Ident;
use redox_lexer::{Keyword, TokenKind};

/// The `deriving (A, B, C)` clause, if one is here — doc 18 §2.4.
///
/// Becomes `#[derive(A, B, C)]`. A *clause of the declaration* rather than an
/// attribute above it, which is why doc 18 §7 retires `derive` as a
/// standalone keyword: there is one place a derive can be written, and it is
/// inside the declaration it belongs to.
///
/// No clause returns an empty vector rather than an error, so a caller can
/// ask unconditionally.
pub(crate) fn deriving(p: &mut Parser) -> Result<Vec<Ident>, ParseError> {
  if !p.check(&TokenKind::Kw(Keyword::Deriving)) {
    return Ok(Vec::new());
  }
  p.advance();
  p.expect(&TokenKind::LParen, "expected `(` after `deriving`")?;

  let mut traits = Vec::new();
  loop {
    if p.check(&TokenKind::RParen) {
      break;
    }
    traits.push(p.ident("expected a trait name")?);
    if !p.check(&TokenKind::Comma) {
      break;
    }
    p.advance();
  }
  p.expect(
    &TokenKind::RParen,
    "expected `)` to close the `deriving` list",
  )?;
  Ok(traits)
}
