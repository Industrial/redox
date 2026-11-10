use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::condition::condition;
use crate::parse::parser::parse_expr::parse_expr;
use redox_ast::{Block, Expr};
use redox_lexer::{Keyword, TokenKind};

/// Parse doc 01 §8's conditional **expression** — `if c then a else b`, plan
/// 20.
///
/// Doc 18 §7 arrives `then` for exactly this construct and nothing else uses
/// the word, which is why there are two surface forms for one Rust construct:
/// the alternative is a reserved keyword with no use.
///
/// Reached only from `prefix`, so the indented form never arrives here —
/// `stmt` asks `control_flow` before it reads an expression at all. That is
/// also what keeps the two spellings apart now that plan 55 leaf 1 has made
/// them one node: **position decides**, and this function is the expression
/// position.
///
/// Each branch becomes a [`Block`] of one element, which is what the node
/// holds since leaf 1 and what the indented form's branches already were.
/// The inline spelling is a spelling, so it is built as the block it means
/// and the printers read it back out of there.
///
/// `is_inline` is what records that it *was* the one-line spelling, and it is
/// not derivable from position: `Parser::inline_body` puts whatever
/// `parse_expr` returns into a block element, so `f x = if c then a else b`
/// is a one-line `if` standing where a statement stands.
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
    condition,
    then_block: Block::of(then),
    otherwise: Some(Block::of(otherwise)),
    is_inline: true,
    span: (start, p.prev_span().1),
  })
}
