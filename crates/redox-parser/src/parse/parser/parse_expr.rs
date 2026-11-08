use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::postfix::postfix;
use crate::parse::parser::prefix::prefix;
use crate::parse::precedence::binary_power::binary_power;
use crate::parse::precedence::postfix_power::POSTFIX_POWER;
use redox_ast::Expr;

/// Parse an expression by Pratt loop, stopping before any operator whose left
/// binding power is below `min`.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) leaf 3. The table in
/// `crate::parse::precedence` is the single place precedence and
/// associativity live, so adding an operator never means touching this loop.
pub(crate) fn parse_expr(p: &mut Parser, min: u8) -> Result<Expr, ParseError> {
  let mut lhs = prefix(p)?;

  loop {
    // Postfix binds tightest, so it is tried before any infix operator.
    if POSTFIX_POWER >= min {
      match postfix(p, lhs)? {
        Ok(grown) => {
          lhs = grown;
          continue;
        }
        Err(unchanged) => lhs = unchanged,
      }
    }

    let Some((op, left, right)) = binary_power(&p.peek().kind) else {
      return Ok(lhs);
    };
    if left < min {
      return Ok(lhs);
    }
    p.advance();
    let rhs = parse_expr(p, right)?;
    let span = (lhs.span().0, rhs.span().1);
    lhs = Expr::Binary {
      op,
      lhs: Box::new(lhs),
      rhs: Box::new(rhs),
      span,
    };
  }
}
