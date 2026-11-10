use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::Expr;
use redox_lexer::Keyword;

/// A block with a modifier in front of it — `try`, `async` or `unsafe`.
///
/// ```text
/// r = try                 work = async            raw = unsafe
///   Ok ((parse x)?)         n <- fetch ()           *p
///                           n + 1
/// ```
///
/// Doc 03 §12.4, §14.3 and plan 45 leaf 2 shipped these as three constructs
/// of one shape, each reached from `prefix` *"exactly as plan 20's `do` is"*.
/// Plan 55 leaf 7 reads them the other way round, which is §1.7's wording:
/// `try` and `async` are **adjectives on a block**, which is what they always
/// were. The block is the construct — [`block_expr`](super::block_expr) — and
/// the word in front of it chooses the lowering.
///
/// Nothing in the surface moved. What this replaces is three arms of `prefix`
/// that each read the same indented block and differed only in which node
/// they built, written when `do` was the thing they were modelled on.
///
/// `unsafe` travels with them even though it is not an adjective in Redox's
/// sense — it is Rust's own keyword and Rust's own block — because the parse
/// is identical and a fourth copy of it would drift from the other three.
pub(crate) fn modified_block(
  p: &mut Parser,
  kw: Keyword,
  start: usize,
) -> Result<Expr, ParseError> {
  p.advance();
  let body = p.statement_block(&format!("expected an indented `{}` body", kw.text()))?;
  let span = (start, p.prev_span().1);
  Ok(match kw {
    Keyword::Async => Expr::AsyncBlock { body, span },
    Keyword::Unsafe => Expr::UnsafeBlock { body, span },
    Keyword::Try => Expr::TryBlock { body, span },
    _ => unreachable!("only `try`, `async` and `unsafe` modify a block"),
  })
}
