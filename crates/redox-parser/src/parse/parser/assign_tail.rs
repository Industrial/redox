use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::compound_assign::compound_assign;
use crate::parse::parser::parse_expr::parse_expr;
use redox_ast::Expr;
use redox_lexer::TokenKind;

/// The assignment that follows a place expression, when one does — doc 01
/// §4.3's compound operators and plan 54's `:=`.
///
/// ```text
/// counter += 1       counter += 1;
/// counter := 2   →   counter = 2;
/// ```
///
/// Read **after** the place expression, because every one of the six tokens
/// can only follow one. A plain `=` is not among them: at body level `x = v`
/// is a *binding*, which `Parser::try_binding` has already taken — and `:=`
/// reaches here rather than there because `looks_like_a_binding` tests for
/// `Eq`, `Colon` or `LBrace` at the second token and `reaches_a_binding_eq`
/// scans for a depth-zero `Eq`. A `ColonEq` answers no to both, so if this
/// were ever missed the failure would be a loud *"expected a line break"*
/// rather than a silent misparse.
///
/// Two positions ask: a block element, and a `case` arm body since plan 55
/// leaf 3 — criterion 3's *"a compound assignment is a legal arm body"*.
/// Shared rather than written twice, so the six operators cannot come to
/// differ by position.
///
/// `start` is where the assignment's span opens, which is the asking
/// construct's first token rather than the target's: a block element's span
/// begins before any `(` the target was wrapped in.
pub(crate) fn assign_tail(p: &mut Parser, target: Expr, start: usize) -> Result<Expr, ParseError> {
  let op = if p.check(&TokenKind::ColonEq) {
    p.advance();
    None
  } else if let Some(op) = compound_assign(p) {
    Some(op)
  } else {
    return Ok(target);
  };

  let value = Box::new(parse_expr(p, 0)?);
  Ok(Expr::Assign {
    target: Box::new(target),
    op,
    value,
    span: (start, p.prev_span().1),
  })
}
