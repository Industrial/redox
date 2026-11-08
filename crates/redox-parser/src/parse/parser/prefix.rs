use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::path_or_ident::path_or_ident;
use crate::parse::parser::unary_op::unary_op;
use crate::parse::precedence::prefix_power::PREFIX_POWER;
use redox_ast::Expr;
use redox_lexer::{Keyword, TokenKind};

/// A prefix position: a literal, a name, a grouping, a receiver sigil, or a
/// unary operator.
pub(crate) fn prefix(p: &mut Parser) -> Result<Expr, ParseError> {
  let start = p.peek_span().0;

  if let Some(op) = unary_op(p) {
    let operand = parse_expr(p, PREFIX_POWER)?;
    let span = (start, operand.span().1);
    return Ok(Expr::Unary {
      op,
      operand: Box::new(operand),
      span,
    });
  }

  match p.peek().kind.clone() {
    TokenKind::Int(text) | TokenKind::Float(text) => {
      let span = p.peek_span();
      p.advance();
      Ok(Expr::Num { text, span })
    }
    // `raw` rather than `parts`: the emitter still passes a simple literal
    // through verbatim, and interpolation is plan 23.
    TokenKind::Str { raw, .. } => {
      let span = p.peek_span();
      p.advance();
      Ok(Expr::StrLit { value: raw, span })
    }
    TokenKind::Char(value) => {
      let span = p.peek_span();
      p.advance();
      Ok(Expr::Char { value, span })
    }
    TokenKind::Kw(kw @ (Keyword::True | Keyword::False)) => {
      let span = p.peek_span();
      p.advance();
      Ok(Expr::Bool {
        value: kw == Keyword::True,
        span,
      })
    }
    // `@field` → `self.field`, `@@name` → `Self::name` (doc 01 §16). Legal
    // only in a method body; plan 25 is where that is checked.
    TokenKind::At => {
      p.advance();
      let name = p.ident("expected a field name after `@`")?;
      let span = (start, name.span.1);
      Ok(Expr::SelfField { name, span })
    }
    TokenKind::AtAt => {
      p.advance();
      let name = p.ident("expected a name after `@@`")?;
      let span = (start, name.span.1);
      Ok(Expr::SelfType { name, span })
    }
    TokenKind::LParen => {
      p.advance();
      let inner = parse_expr(p, 0)?;
      p.expect(&TokenKind::RParen, "expected `)` to close the group")?;
      Ok(Expr::Paren {
        inner: Box::new(inner),
        span: (start, p.prev_span().1),
      })
    }
    TokenKind::Ident(_) => path_or_ident(p),
    _ => Err(ParseError {
      message: "expected an expression".to_string(),
      span: p.peek_span(),
    }),
  }
}
