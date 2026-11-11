use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::Ident;
use redox_lexer::{Keyword, TokenKind};

/// The `deriving (A, B, C)` clause, if one is here — doc 18 §2.4.
///
/// Becomes `#[derive(A, B, C)]`, and an entry may be a **path** —
/// `deriving (Debug, serde::Deserialize)` is `#[derive(Debug,
/// serde::Deserialize)]`.
/// A *clause of the declaration* rather than an
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
    p.skip_layout();
    if p.check(&TokenKind::RParen) {
      break;
    }
    // **A path, not just a name.** `#[derive(serde::Deserialize)]` is
    // ordinary Rust, and this clause emits straight into that attribute — it
    // simply could not read one. Nine of `redox-mcp`'s nineteen files use the
    // qualified form, and plan 77 is what found it.
    //
    // The segments are joined into the `Ident`'s text, which `emit::data`
    // writes verbatim, so nothing downstream has to know.
    let mut name = p.ident("expected a trait name")?;
    while p.check(&TokenKind::ColonColon) {
      p.advance();
      let segment = p.ident("expected a name after `::`")?;
      name.text.push_str("::");
      name.text.push_str(&segment.text);
      name.span.1 = segment.span.1;
    }
    traits.push(name);
    p.skip_layout();
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
