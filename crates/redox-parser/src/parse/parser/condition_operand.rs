use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::above_and_and::ABOVE_AND_AND;
use crate::parse::parser::holds_a_draw_arrow::holds_a_draw_arrow;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::pattern_alt::pattern_alt;
use redox_ast::ConditionOperand;
use redox_lexer::TokenKind;

/// One operand: a pattern test if a `<-` reaches this far, else an expression.
pub(crate) fn condition_operand(p: &mut Parser) -> Result<ConditionOperand, ParseError> {
  // This scan stops at `&&`, because the question here is whether *this*
  // operand is a pattern test and not whether a later one is.
  if !holds_a_draw_arrow(p, true) {
    return Ok(ConditionOperand::Expr(parse_expr(p, ABOVE_AND_AND)?));
  }
  let pattern = pattern_alt(p)?;
  p.expect(
    &TokenKind::DrawArrow,
    "expected `<-` after the pattern — doc 18 §3 writes a pattern test with the draw arrow",
  )?;
  Ok(ConditionOperand::Pattern {
    pattern,
    source: parse_expr(p, ABOVE_AND_AND)?,
  })
}
