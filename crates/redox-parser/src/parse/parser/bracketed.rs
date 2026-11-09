use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::qualifiers::qualifiers;
use crate::parse::precedence::above_bitor::above_bitor;
use redox_ast::Expr;
use redox_lexer::TokenKind;

/// Parse what a `[` opens: a list literal or doc 18 §3.2's comprehension.
///
/// ```text
/// [1, 2, 3]                    a list literal — Rust's array literal
/// [ x * y | x in 1..3, y in ys, x != y ]   a comprehension
/// []                           the empty list
/// ```
///
/// One token of lookahead after the first expression settles it: a `|` opens
/// the qualifier list, a `,` or a `]` closes a literal. The two share their
/// brackets, and reading either without the other would have left `[`
/// ambiguous — which is why they arrive together.
pub(crate) fn bracketed(p: &mut Parser) -> Result<Expr, ParseError> {
  let start = p.peek_span().0;
  p.expect(&TokenKind::LBracket, "expected `[`")?;

  if p.check(&TokenKind::RBracket) {
    p.advance();
    return Ok(Expr::List {
      items: Vec::new(),
      span: (start, p.prev_span().1),
    });
  }

  // Above bitwise or, so a bare `|` is left for the qualifier list rather
  // than absorbed as an operator — doc 18 §5.4's rule, applied to the third
  // place the bar is structural.
  let first = parse_expr(p, above_bitor())?;

  if p.check(&TokenKind::Or) {
    p.advance();
    let quals = qualifiers(p)?;
    p.expect(
      &TokenKind::RBracket,
      "expected `]` to close the comprehension",
    )?;
    return Ok(Expr::Comprehension {
      element: Box::new(first),
      qualifiers: quals,
      span: (start, p.prev_span().1),
    });
  }

  // Doc 03 §10.4's array repeat — plan 29. A `;` after the first element
  // means repeat; a `,` means list. One token of lookahead, no backtracking,
  // and the comprehension's `|` was already ruled out above.
  if p.check(&TokenKind::Semi) {
    p.advance();
    let count = parse_expr(p, 0)?;
    p.expect(
      &TokenKind::RBracket,
      "expected `]` to close the array repeat",
    )?;
    return Ok(Expr::ArrayRepeat {
      value: Box::new(first),
      count: Box::new(count),
      span: (start, p.prev_span().1),
    });
  }

  let mut items = vec![first];
  while p.check(&TokenKind::Comma) {
    p.advance();
    if p.check(&TokenKind::RBracket) {
      break;
    }
    items.push(parse_expr(p, 0)?);
  }
  p.expect(&TokenKind::RBracket, "expected `]` to close the list")?;
  Ok(Expr::List {
    items,
    span: (start, p.prev_span().1),
  })
}
