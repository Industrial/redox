use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::parse_expr::parse_expr;
use redox_ast::FieldInit;
use redox_lexer::TokenKind;

/// One `name = value` field of a record literal, or a punned `name`.
///
/// Doc 18 §2.4. `=` for a value, against `::` for a type — one token
/// introduces a type wherever it appears, the other introduces a value, in a
/// declaration and a literal alike.
///
/// Punning is recorded as an **absent** value rather than a synthesised
/// `Expr::Ident`, so the formatter can write back what the author wrote.
pub(crate) fn field_init(p: &mut Parser) -> Result<FieldInit, ParseError> {
  let start = p.peek_span().0;
  let name = p.ident("expected a field name")?;

  let value = if p.check(&TokenKind::Eq) {
    p.advance();
    Some(parse_expr(p, 0)?)
  } else {
    None
  };

  Ok(FieldInit {
    name,
    value,
    span: (start, p.prev_span().1),
  })
}
