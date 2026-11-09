use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::pattern_alt::pattern_alt;
use crate::parse::parser::reaches_a_draw_arrow::reaches_a_draw_arrow;
use redox_ast::StmtKind;
use redox_lexer::TokenKind;

/// An await-bind, if the cursor is on one — doc 30, the whole of it.
///
/// ```text
/// user <- fetch id          let user = fetch(id).await;
/// <- flush ()           →   flush().await;
/// (a, b) <- load id         let (a, b) = load(id).await;
/// ```
///
/// **`<-` means this and nothing else.** It is Haskell's monadic bind for the
/// monad Redox actually has: `do` notation binds an effect's result to a name,
/// and `x <- fetch id` binds a future's result to a name. The monad differs
/// and the reading does not — which is doc 30 §1.1's whole argument, and why
/// doc 18 §3.1's four positions were given other spellings first.
///
/// # Every await binds, and the binder may be nothing
///
/// `<- flush ()` awaits a future for its effect and binds nothing, which is
/// the one form with no pattern — spelled by having nothing before the arrow,
/// because `_ <- f` would be a pattern that discards where no pattern is
/// wanted. Rust writes `f().await;` and so does this.
///
/// # Why a scan rather than one token of lookahead
///
/// The left side is a **pattern** (plan 33), so `(a, b) <- load id` and
/// `Point { x } <- fetch ()` are both legal and neither is one token long.
/// Finding the `<-` means scanning to it at depth zero, which is the same
/// shape `looks_like_a_binding` uses for the same reason — and a lookahead
/// that advances nothing cannot leave the parser somewhere unexpected when it
/// answers no.
pub(crate) fn try_await(p: &mut Parser) -> Result<Option<StmtKind>, ParseError> {
  // The effect-only form: nothing to bind, so nothing to scan past.
  if p.check(&TokenKind::DrawArrow) {
    p.advance();
    return Ok(Some(StmtKind::Await {
      pattern: None,
      value: parse_expr(p, 0)?,
    }));
  }

  if !reaches_a_draw_arrow(p) {
    return Ok(None);
  }

  let pattern = pattern_alt(p)?;
  p.expect(&TokenKind::DrawArrow, "expected `<-`")?;
  Ok(Some(StmtKind::Await {
    pattern: Some(pattern),
    value: parse_expr(p, 0)?,
  }))
}
