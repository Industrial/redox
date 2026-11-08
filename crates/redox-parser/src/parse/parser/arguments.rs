use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::parse_expr::parse_expr;
use redox_ast::Expr;
use redox_lexer::TokenKind;

/// A comma-separated argument list, up to but not including `close`.
///
/// A trailing comma is accepted, because the lexer's continuation suppression
/// (plan 02 leaf 2) makes a multi-line argument list natural to write.
pub(crate) fn arguments(p: &mut Parser, close: &TokenKind) -> Result<Vec<Expr>, ParseError> {
  let mut args = Vec::new();
  while !p.check(close) && !p.at_eof() {
    args.push(parse_expr(p, 0)?);
    if !p.check(&TokenKind::Comma) {
      break;
    }
    p.advance();
  }
  Ok(args)
}
