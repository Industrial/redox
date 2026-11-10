use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::above_and_and::ABOVE_AND_AND;
use crate::parse::parser::holds_a_pattern_test::holds_a_pattern_test;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::pattern_alt::pattern_alt;
use redox_ast::ConditionOperand;
use redox_lexer::{Keyword, TokenKind};

/// One operand: a pattern test if a `<-` reaches this far, else an expression.
pub(crate) fn condition_operand(p: &mut Parser) -> Result<ConditionOperand, ParseError> {
  // This scan stops at `&&`, because the question here is whether *this*
  // operand is a pattern test and not whether a later one is.
  if !holds_a_pattern_test(p, true, &TokenKind::Kw(Keyword::Is)) {
    return Ok(ConditionOperand::Expr(parse_expr(p, ABOVE_AND_AND)?));
  }
  // Plan 60: **scrutinee first.** `slot is Some n`, which is the order a
  // reader thinks in and the order Rust, C# and Kotlin all use. Doc 30 §7
  // decision 3 chose `=` partly to avoid reversing the operands; this row
  // takes the reversal as the improvement rather than the price.
  //
  // The node is unchanged, so `emit_condition` and every lowering are
  // untouched — only the parse order and the token moved.
  let source = parse_expr(p, ABOVE_AND_AND)?;
  p.expect(
    &TokenKind::Kw(Keyword::Is),
    "expected `is` before the pattern",
  )?;
  Ok(ConditionOperand::Pattern {
    pattern: pattern_alt(p)?,
    source,
  })
}
