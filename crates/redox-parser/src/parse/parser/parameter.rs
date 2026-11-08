use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::Param;
use redox_lexer::{Keyword, TokenKind};

/// One parameter: an optional `mut`, a name, and an optional `: Type`.
///
/// A sibling of `parse_signature` rather than a child because doc 06 §2.4 caps
/// depth at three levels below `src/`.
pub(crate) fn parameter(p: &mut Parser) -> Result<Param, ParseError> {
  let start = p.peek_span().0;
  let is_mut = p.eat_keyword(Keyword::Mut);
  let name = p.ident("expected a parameter name")?;

  let ty = if p.check(&TokenKind::Colon) {
    p.advance();
    Some(p.type_expr("expected a type after `:`")?)
  } else {
    None
  };

  Ok(Param {
    name,
    ty,
    is_mut,
    span: (start, p.prev_span().1),
  })
}
