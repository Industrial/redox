use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::condition_operand::condition_operand;
use crate::parse::parser::holds_a_pattern_test::holds_a_pattern_test;
use crate::parse::parser::parse_expr::parse_expr;
use redox_ast::Condition;
use redox_diag::Code;
use redox_lexer::TokenKind;

/// An `if`, `while` or arm-guard condition — plan 22, doc 18 §3's third and
/// fourth positions for `<-`.
///
/// ```text
/// if n > 0                     if n > 0 {
/// if Some v <- x               if let Some(v) = x {
/// if Some v <- x && v > 2      if let Some(v) = x && v > 2 {
/// ```
///
/// # The pattern-free condition takes the path it always took
///
/// A condition containing no `<-` is parsed by one `parse_expr(p, 0)` call —
/// byte for byte what plan 20 did. This matters for more than speed: parsing
/// operand-wise changes what the precedence climber sees, and `a && b || c` as
/// a chain of two operands would leave `|| c` unread. So the scan below is not
/// an optimisation, it is the thing that keeps every existing condition
/// correct, and criterion 8 asserts it.
///
/// The scan is **lookahead, not backtracking**: it reads token kinds and
/// advances nothing.
pub(crate) fn condition(p: &mut Parser) -> Result<Condition, ParseError> {
  // Does the **whole** condition hold a pattern test? This scan runs past
  // `&&`, unlike the per-operand one below: a chain's pattern test may be any
  // operand, and `if n > 0 && Some v <- x` is as legal as `if Some v <- x`.
  if !holds_a_pattern_test(p, false) {
    return Ok(Condition::plain(parse_expr(p, 0)?));
  }

  let mut operands = vec![condition_operand(p)?];
  while p.check(&TokenKind::AndAnd) {
    p.advance();
    operands.push(condition_operand(p)?);
  }

  // A let-chain joins with `&&` only — **Rust's** restriction, not Redox's —
  // so say that rather than leaving the generic "expected a line break" the
  // block parser would give.
  if p.check(&TokenKind::OrOr) {
    return Err(ParseError::with(
      Code::ExpectedToken,
      "a condition holding a pattern test joins with `&&` only: Rust's let-chains do \
       not admit `||`. Split the condition, or use a `case`",
      p.peek_span(),
    ));
  }
  Ok(Condition { operands })
}
