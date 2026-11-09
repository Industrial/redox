use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::case_of::case_of;
use crate::parse::parser::lambda::lambda;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::path_or_ident::path_or_ident;
use crate::parse::parser::section::section;
use crate::parse::parser::unary_op::unary_op;
use crate::parse::precedence::prefix_power::PREFIX_POWER;
use redox_ast::Expr;
use redox_diag::Code;
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
      // Doc 18 §4's sections. Tried before the grouping, because `(> 0)` is
      // not an expression at all — and `section` puts the cursor back when
      // what follows turns out to be an ordinary group.
      if let Some(found) = section(p, start)? {
        return Ok(found);
      }
      let inner = parse_expr(p, 0)?;
      p.expect(&TokenKind::RParen, "expected `)` to close the group")?;
      Ok(Expr::Paren {
        inner: Box::new(inner),
        span: (start, p.prev_span().1),
      })
    }
    // Doc 18 §4's lambda, and `move \x -> …` with it: `move` can only open
    // a closure here, so one token of lookahead settles it.
    TokenKind::Backslash => lambda(p),
    TokenKind::Kw(Keyword::Move) => lambda(p),
    // Doc 18 §2.3's `case e of`. An expression, not a statement: its value is
    // the matching arm's, which is what lets it be a clause body, an argument
    // or the right-hand side of a binding with no extra form.
    TokenKind::Kw(Keyword::Case) => case_of(p),
    // `self` and `Self` are expressions, because doc 18 §2.5 drops `@name`
    // and `@@name` in favour of `self.name` and `Self::name`. They lex as
    // reserved words, so they need naming here — an `Ident` arm would never
    // see them.
    TokenKind::Kw(Keyword::SelfValue) => {
      let span = p.peek_span();
      p.advance();
      Ok(Expr::Ident(redox_ast::Ident {
        text: "self".to_string(),
        span,
      }))
    }
    TokenKind::Kw(Keyword::SelfType) => {
      let span = p.peek_span();
      p.advance();
      Ok(Expr::Ident(redox_ast::Ident {
        text: "Self".to_string(),
        span,
      }))
    }
    TokenKind::Ident(_) => path_or_ident(p),
    _ => Err(ParseError::with(
      Code::ExpectedExpression,
      "expected an expression",
      p.peek_span(),
    )),
  }
}
