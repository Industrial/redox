use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::application::application;
use crate::parse::parser::capture_type::capture_type;
use crate::parse::parser::type_stop::TypeStop;
use crate::parse::precedence::binary_power::binary_power;
use redox_ast::Expr;
use redox_lexer::TokenKind;

/// Parse an expression by Pratt loop, stopping before any operator whose left
/// binding power is below `min`.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) leaf 3, with doc 18 §2.1's
/// application as its primary. The table in
/// `crate::parse::precedence` is the single place precedence and
/// associativity live, so adding an operator never means touching this loop.
pub(crate) fn parse_expr(p: &mut Parser, min: u8) -> Result<Expr, ParseError> {
  // Doc 18 §2.1: application binds tighter than every operator, so it is the
  // primary rather than a rung on the precedence ladder. `application` also
  // absorbs the postfix loop that used to run here, because an argument is a
  // postfixed primary and the head is one too.
  let mut lhs = application(p)?;

  loop {
    // Doc 18 §6's ascription, at the loosest binding power there is: it
    // applies to the whole expression, so it is tried only at the top level
    // (`min == 0`) and only after the binary loop has taken everything else.
    // `1 + 2 : u64` therefore ascribes the sum, not the `2`.
    if min == 0 && p.check(&TokenKind::Colon) {
      p.advance();
      let ty = capture_type(p, "expected a type after `:`", TypeStop::Greedy)?;
      let span = (lhs.span().0, ty.span.1);
      lhs = Expr::Ascribe {
        inner: Box::new(lhs),
        ty,
        span,
      };
      continue;
    }

    // Doc 18 §2.1's `$`: grouping that runs to the end of the expression, and
    // the reason a nested application rarely needs brackets. It **lowers to
    // nothing at all** — `f $ g x` is the call `f(g(x))` — so the call is
    // built here and no node of its own is carried.
    //
    // `is_dollar` records the spelling and nothing else. The lowering ignores
    // it, because `$` has no lowering; the formatter reads it, because a
    // formatter that reprinted `f $ x` as `f (x)` would rewrite away the one
    // construct whose purpose is to avoid that parenthesis.
    //
    // Binding power 1: looser than every operator, tighter than ascription,
    // which matches Haskell's `infixr 0` for `$` against an even looser `:`.
    if min <= 1 && p.check(&TokenKind::Dollar) {
      p.advance();
      let arg = parse_expr(p, 1)?;
      let span = (lhs.span().0, arg.span().1);
      lhs = Expr::Call {
        callee: Box::new(lhs),
        args: vec![arg],
        is_dollar: true,
        span,
      };
      continue;
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
