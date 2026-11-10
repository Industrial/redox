use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::assign_tail::assign_tail;
use crate::parse::parser::parse_expr::parse_expr;
use redox_ast::Expr;

/// A `case` arm's body, after its `->` — plan 55 leaf 3.
///
/// ```text
/// case score of                   match score {
///   0 ->                              0 => {
///     base = "none"           →           let base = "none";
///     base                                base
///   _ -> "some"                       }
///                                     _ => "some",
///                                 }
/// ```
///
/// An arm body is an **expression**, and an indented block is one — so the
/// block needs no marker in front of it.
///
/// **Leaf 7 deleted this function's block case rather than its keyword.**
/// Leaf 3 read the block here, because `prefix` had no production for a bare
/// one and `-> do` was still accepted; leaf 7 gave `prefix` the production
/// (`block_expr`), so `parse_expr` reaches it on its own and §10.10's
/// prediction holds — *"once `prefix` parses a bare indented block,
/// `arm_body`'s special case simply goes"*.
///
/// What is left is the assignment tail, which is criterion 3's second half:
/// `Exit::Success _ -> enriched += 1` was `RDX0101` before leaf 3, because
/// `parse_expr` stops at `+=` and only `Parser::stmt` read what follows.
/// `assign_tail` is that reading, shared — and it declines harmlessly on a
/// block, which ends on a `Dedent` with no operator after it.
pub(crate) fn arm_body(p: &mut Parser) -> Result<Expr, ParseError> {
  let start = p.peek_span().0;
  let value = parse_expr(p, 0)?;
  assign_tail(p, value, start)
}
