use crate::parse::parser::Parser;
use crate::parse::precedence::binary_power::binary_power;
use redox_ast::BinaryOp;

/// The binary operator at the cursor, consumed — doc 18 §4.
///
/// Reads the operator **without** its precedence, which a section does not
/// need: a section's operand is already delimited by the parentheses, so
/// there is nothing to bind more or less tightly than.
///
/// Goes through `binary_power` rather than a second table, so a new operator
/// cannot be added to the expression grammar and forgotten here.
pub(crate) fn binary_op(p: &mut Parser) -> Option<BinaryOp> {
  let (op, _, _) = binary_power(&p.peek().kind)?;
  p.advance();
  Some(op)
}
