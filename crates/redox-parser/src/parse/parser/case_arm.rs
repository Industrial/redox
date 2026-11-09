use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::pattern_alt::pattern_alt;
use redox_ast::CaseArm;
use redox_lexer::TokenKind;

/// One `pattern [| guard] -> body` arm of a `case`.
///
/// Doc 18 §5.4: the `|` here is **the guard**, not alternation, and the only
/// reason that is unambiguous is that alternation keeps doc 03's `or` keyword
/// — `Teen or PreTeen -> …` cannot be misread, `Teen | PreTeen -> …` could.
///
/// One guard, not a ladder. An equation's guards are a list because Rust has
/// no equation form to lower them into; an arm's guard is a single `if`
/// because Rust's `match` arm already has exactly that, and doc 18 §1 holds
/// the lowering fixed wherever Rust has the shape already.
pub(crate) fn case_arm(p: &mut Parser) -> Result<CaseArm, ParseError> {
  let start = p.peek_span().0;
  let pattern = pattern_alt(p)?;

  let guard = if p.check(&TokenKind::Or) {
    p.advance();
    Some(parse_expr(p, 0)?)
  } else {
    None
  };

  p.expect(&TokenKind::Arrow, "expected `->` after a `case` pattern")?;
  let body = parse_expr(p, 0)?;
  p.newline_with_trailing("expected a line break after a `case` arm")?;

  Ok(CaseArm {
    pattern,
    guard,
    body,
    span: (start, p.prev_span().1),
  })
}
