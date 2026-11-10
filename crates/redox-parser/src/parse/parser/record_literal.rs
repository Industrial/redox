use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::field_init::field_init;
use redox_ast::Expr;
use redox_lexer::TokenKind;

/// Parse `head { name = value, … }` — doc 18 §2.4's record literal.
///
/// ```text
/// Point { x = 10, y = 20 }    construction
/// Point { x, y }              punning
/// p { x = 99 }                update
/// ```
///
/// Called from `postfix`, so the head is whatever expression preceded the
/// brace. That is what makes update fall out of construction rather than
/// needing a form of its own: both are `head { fields }`, and the *emitter*
/// decides which by the case of the head (doc 18 §5.3).
///
/// A `{` after an expression is unambiguous here because Redox has no
/// brace-delimited block in expression position — layout does that job — so
/// nothing else a `{` could open is in the grammar.
pub(crate) fn record_literal(p: &mut Parser, head: Expr) -> Result<Expr, ParseError> {
  let start = head.span().0;
  p.expect(&TokenKind::LBrace, "expected `{` to open a record")?;

  let mut fields = Vec::new();
  loop {
    p.skip_layout();
    if p.check(&TokenKind::RBrace) {
      break;
    }
    fields.push(field_init(p)?);
    p.skip_layout();
    if !p.check(&TokenKind::Comma) {
      break;
    }
    p.advance();
  }
  p.skip_layout();
  p.expect(&TokenKind::RBrace, "expected `}` to close the record")?;

  Ok(Expr::Record {
    head: Box::new(head),
    fields,
    span: (start, p.prev_span().1),
  })
}
