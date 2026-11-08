use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::parameter::parameter;
use redox_ast::Signature;
use redox_lexer::TokenKind;

/// Parse a signature: an optional parameter list and an optional return type.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) leaf 4. Both parts are optional —
/// doc 01 §5.1 makes the return type optional, and a zero-parameter function
/// writes no list at all (`main = ->`), which is the form plan 01 shipped and
/// which must keep parsing unchanged.
pub(crate) fn parse_signature(p: &mut Parser) -> Result<Signature, ParseError> {
  let start = p.peek_span().0;
  let mut sig = Signature::default();

  if p.check(&TokenKind::LParen) {
    p.advance();
    while !p.check(&TokenKind::RParen) && !p.at_eof() {
      sig.params.push(parameter(p)?);
      if !p.check(&TokenKind::Comma) {
        break;
      }
      p.advance();
    }
    p.expect(
      &TokenKind::RParen,
      "expected `)` to close the parameter list",
    )?;
  }

  if p.check(&TokenKind::Colon) {
    p.advance();
    sig.ret = Some(p.type_expr("expected a return type after `:`")?);
  }

  sig.span = (start, p.prev_span().1);
  Ok(sig)
}
