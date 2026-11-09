use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::condition::condition;
use crate::parse::parser::parse_expr::parse_expr;
use redox_ast::Expr;
use redox_lexer::{Keyword, TokenKind};

/// Parse doc 01 §8's conditional **expression** — `if c then a else b`, plan
/// 20.
///
/// Doc 18 §7 arrives `then` for exactly this construct and nothing else uses
/// the word, which is why there are two surface forms for one Rust construct:
/// the alternative is a reserved keyword with no use.
///
/// Reached only from `prefix`, so the statement form never arrives here —
/// `stmt` asks `control_flow` before it reads an expression at all.
///
/// The `else` is **required**. Rust's `if` without one has type `()`, so a
/// conditional expression that omitted it could only ever be a statement, and
/// the statement form is the indented one.
pub(crate) fn if_expression(p: &mut Parser) -> Result<Expr, ParseError> {
  let start = p.peek_span().0;
  p.expect(&TokenKind::Kw(Keyword::If), "expected `if`")?;
  let condition = condition(p)?;
  p.expect(
    &TokenKind::Kw(Keyword::Then),
    "expected `then` — an `if` without it is the indented statement form",
  )?;
  let then = parse_expr(p, 0)?;
  p.expect(
    &TokenKind::Kw(Keyword::Else),
    "expected `else` — a conditional expression must have one, because Rust's `if` without an `else` has type `()`",
  )?;
  let otherwise = parse_expr(p, 0)?;

  Ok(Expr::If {
    condition: Box::new(condition),
    then: Box::new(then),
    otherwise: Box::new(otherwise),
    span: (start, p.prev_span().1),
  })
}
