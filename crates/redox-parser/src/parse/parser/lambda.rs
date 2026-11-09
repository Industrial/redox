use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::capture_type::capture_type;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::spaced_colon_colon::spaced_colon_colon;
use crate::parse::parser::type_stop::TypeStop;
use redox_ast::{Expr, Param};
use redox_lexer::{Keyword, TokenKind};

/// Parse doc 18 §4's lambda: `\x -> e`, optionally `move` and typed.
///
/// ```text
/// \n -> n * n              |n| n * n
/// \x :: i32 -> x + 1       |x: i32| x + 1
/// move \x -> x + captured  move |x| x + captured
/// \ -> 0                   || 0
/// ```
///
/// The binder's type uses the **same spaced `::`** as a signature line and an
/// ascription (doc 18 §5.2), which is why the type is captured with
/// `capture_type` rather than a second notation: there is one way to write a
/// type in Redox and this is it.
///
/// Doc 18 §8 declines currying, so a lambda's parameters are a list rather
/// than nested single-parameter lambdas — `\x y -> e` is one closure of two
/// arguments, matching how application works (§2.1).
pub(crate) fn lambda(p: &mut Parser) -> Result<Expr, ParseError> {
  let start = p.peek_span().0;
  let is_move = p.eat_keyword(Keyword::Move);
  p.expect(&TokenKind::Backslash, "expected `\\` to open a lambda")?;

  let mut params = Vec::new();
  while !p.check(&TokenKind::Arrow) && !p.at_eof() {
    let name = p.ident("expected a lambda parameter name")?;
    let ty = if spaced_colon_colon(p) {
      p.advance();
      Some(capture_type(
        p,
        "expected a type after `::`",
        TypeStop::Greedy,
      )?)
    } else {
      None
    };
    let span = name.span;
    params.push(Param {
      name,
      ty,
      is_mut: false,
      span,
    });
  }

  p.expect(
    &TokenKind::Arrow,
    "expected `->` after the lambda's parameters",
  )?;
  let body = parse_expr(p, 0)?;
  let span = (start, body.span().1);
  Ok(Expr::Lambda {
    params,
    body: Box::new(body),
    is_move,
    span,
  })
}
