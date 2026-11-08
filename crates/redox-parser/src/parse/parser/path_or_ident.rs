use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::Expr;
use redox_lexer::TokenKind;

/// An identifier, or a `::`-joined path carried verbatim.
///
/// Doc 03 §13.1 makes `::` the only path separator, so `.` unambiguously means
/// field or method access — which is what lets `postfix` treat every `.` the
/// same way.
pub(crate) fn path_or_ident(p: &mut Parser) -> Result<Expr, ParseError> {
  let first = p.ident("expected a name")?;
  if !p.check(&TokenKind::ColonColon) {
    return Ok(Expr::Ident(first));
  }

  let start = first.span.0;
  let mut text = first.text;
  while p.check(&TokenKind::ColonColon) {
    p.advance();
    let next = p.ident("expected a name after `::`")?;
    text.push_str("::");
    text.push_str(&next.text);
  }
  Ok(Expr::Path {
    text,
    span: (start, p.prev_span().1),
  })
}
