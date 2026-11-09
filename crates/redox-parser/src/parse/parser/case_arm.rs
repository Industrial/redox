use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::condition::condition;
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

  // An arm guard may be a pattern test as of plan 22 — doc 18 §3's fourth
  // position, *"in a condition or guard"*. It lowers to `if let` inside the
  // match guard, which is `if_let_guard` and **stable** as of rustc 1.97.1 —
  // verified before the plan was written, because a plan that promises an
  // unstable lowering cannot ship (plan 22 §1.5).
  let guard = if p.check(&TokenKind::Or) {
    p.advance();
    Some(condition(p)?)
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
