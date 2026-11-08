use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::starts_argument::starts_argument;
use redox_ast::Expr;
use redox_lexer::TokenKind;

/// A **statement-initial parenless call**, if the cursor is on one.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`). Plan 01 accepted only
/// `name "string"`; this accepts `name expr, expr`, which is what doc 01
/// §31 Phase 1's acceptance test and every `print` of a computed value need.
///
/// Restricted to **statement position**, which is Emerald's plan-90 option (A)
/// — the precedent its own grammar already used for leading unary operators.
/// Plan 19 lifts it to argument position, which is the case that needs doc 03
/// §5.1's whitespace-adjacency lexing rather than a position rule.
///
/// An argument may begin only with a token that is unambiguously
/// expression-starting and never infix. `-`, `!`, `&` and `*` are excluded on
/// purpose: `a - b` as a statement would otherwise read as `a(-b)`, and
/// telling those apart is exactly what plan 19's adjacency rule is for.
pub(crate) fn stmt_call(p: &mut Parser) -> Result<Option<Expr>, ParseError> {
  if !matches!(p.peek().kind, TokenKind::Ident(_)) {
    return Ok(None);
  }
  if !p.peek_at(1).is_some_and(|t| starts_argument(&t.kind)) {
    return Ok(None);
  }

  let callee = p.ident("expected a name")?;
  let start = callee.span.0;
  let mut args = vec![parse_expr(p, 0)?];
  while p.check(&TokenKind::Comma) {
    p.advance();
    args.push(parse_expr(p, 0)?);
  }
  Ok(Some(Expr::Call {
    callee: Box::new(Expr::Ident(callee)),
    args,
    span: (start, p.prev_span().1),
  }))
}
