use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::arguments::arguments;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::record_literal::record_literal;
use redox_ast::{Expr, Ident};
use redox_lexer::TokenKind;

/// One postfix step: a call, an index, a field, or a method call.
///
/// `Ok` means the expression grew; `Err` hands it back unchanged so the Pratt
/// loop can move on to infix operators. The nested `Result` is the price of
/// expressing "no step applied" without a sentinel.
#[allow(clippy::result_large_err)]
pub(crate) fn postfix(p: &mut Parser, lhs: Expr) -> Result<Result<Expr, Expr>, ParseError> {
  let start = lhs.span().0;

  // **No paren-call step.** Doc 18 §2.1 makes parentheses grouping only:
  // `f(x)`, `f (x)` and `f x` are one expression, and `application` builds
  // the call from juxtaposition. A postfix call here would take the
  // parenthesised group as an argument list and leave any following argument
  // to form a *second* call — `map (\n -> n * n) xs` emitted
  // `map(|n| n * n)(xs)` until this was removed.
  //
  // A method call keeps its parentheses, because `.name(args)` is a postfix
  // step on a *receiver* and has no juxtaposition reading to collide with.

  // `e?` — Rust's try operator. A postfix step like any other, and the
  // reason doc 03 §12.5's generated `From` impl has a caller.
  if p.check(&TokenKind::Question) {
    p.advance();
    return Ok(Ok(Expr::Try {
      inner: Box::new(lhs),
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

  // Doc 18 §2.4's record literal. A `{` after an expression is unambiguous:
  // Redox has no brace-delimited block in expression position — layout does
  // that job — so nothing else a `{` could open is in the grammar.
  if p.check(&TokenKind::LBrace) {
    return Ok(Ok(record_literal(p, lhs)?));
  }

  if p.check(&TokenKind::Dot) {
    p.advance();

    // `m.0` — a tuple field. Doc 18 §2.4's `newtype Meters = Meters f64`
    // lowers to a tuple struct, and a tuple struct whose field cannot be read
    // is a declaration with no use. The index is an `Ident` carrying digits
    // rather than a variant of its own, because `Expr::Field` already holds
    // exactly "a name after a dot" and the emitter writes it through
    // unchanged.
    if let TokenKind::Int(digits) = &p.peek().kind {
      let name = Ident {
        text: digits.clone(),
        span: p.peek_span(),
      };
      p.advance();
      return Ok(Ok(Expr::Field {
        recv: Box::new(lhs),
        name,
        span: (start, p.prev_span().1),
      }));
    }

    let name = p.ident("expected a field name, a tuple index, or a method name after `.`")?;
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
