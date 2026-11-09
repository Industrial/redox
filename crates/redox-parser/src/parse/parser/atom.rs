use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::postfix::postfix;
use crate::parse::parser::prefix::prefix;
use redox_ast::Expr;

/// One argument-sized expression: a primary with its postfix operators.
///
/// A sibling of `application` rather than a child, because doc 06 §2.4 caps
/// depth at three levels below `src/` — the same resolution plan 03 reached
/// for `starts_argument`.
///
/// Doc 18 §2.1. An argument is **never** a binary expression, and that is
/// what makes application bind tighter than every operator without consulting
/// the precedence table at all. `add 1 + 2` is `add(1) + 2`, which is
/// Haskell's reading too.
///
/// Postfix is included so `(parse x)?` and `(get u).await` attach to the right
/// thing — doc 18 §2.1's one surviving required-parens case besides nested
/// application.
pub(crate) fn atom(p: &mut Parser) -> Result<Expr, ParseError> {
  let mut expr = prefix(p)?;
  loop {
    match postfix(p, expr)? {
      Ok(grown) => expr = grown,
      Err(unchanged) => return Ok(unchanged),
    }
  }
}
