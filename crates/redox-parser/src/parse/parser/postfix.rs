use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::arguments::arguments;
use crate::parse::parser::parse_expr::parse_expr;
use redox_ast::Expr;
use redox_lexer::TokenKind;

/// One postfix step: a call, an index, a field, or a method call.
///
/// `Ok` means the expression grew; `Err` hands it back unchanged so the Pratt
/// loop can move on to infix operators. The nested `Result` is the price of
/// expressing "no step applied" without a sentinel.
#[allow(clippy::result_large_err)]
pub(crate) fn postfix(p: &mut Parser, lhs: Expr) -> Result<Result<Expr, Expr>, ParseError> {
  let start = lhs.span().0;

  if p.check(&TokenKind::LParen) {
    p.advance();
    let args = arguments(p, &TokenKind::RParen)?;
    p.expect(
      &TokenKind::RParen,
      "expected `)` to close the argument list",
    )?;
    return Ok(Ok(Expr::Call {
      callee: Box::new(lhs),
      args,
      span: (start, p.prev_span().1),
    }));
  }

  if p.check(&TokenKind::LBracket) {
    p.advance();
    let index = parse_expr(p, 0)?;
    p.expect(&TokenKind::RBracket, "expected `]` to close the index")?;
    return Ok(Ok(Expr::Index {
      recv: Box::new(lhs),
      index: Box::new(index),
      span: (start, p.prev_span().1),
    }));
  }

  if p.check(&TokenKind::Dot) {
    p.advance();
    let name = p.ident("expected a field or method name after `.`")?;
    // A `(` immediately after the name makes it a method call; otherwise it is
    // a field. Doc 03 §5.2 keeps the parenless zero-argument form on the `!`
    // sigil, and plan 19 decides the general case.
    if p.check(&TokenKind::LParen) {
      p.advance();
      let args = arguments(p, &TokenKind::RParen)?;
      p.expect(
        &TokenKind::RParen,
        "expected `)` to close the argument list",
      )?;
      return Ok(Ok(Expr::MethodCall {
        recv: Box::new(lhs),
        name,
        args,
        span: (start, p.prev_span().1),
      }));
    }
    return Ok(Ok(Expr::Field {
      recv: Box::new(lhs),
      name,
      span: (start, p.prev_span().1),
    }));
  }

  Ok(Err(lhs))
}
