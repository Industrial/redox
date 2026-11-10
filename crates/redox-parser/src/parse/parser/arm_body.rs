use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::assign_tail::assign_tail;
use crate::parse::parser::parse_expr::parse_expr;
use redox_ast::Expr;
use redox_lexer::TokenKind;

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
/// block needs no marker in front of it. The node is `Expr::Do` with no
/// label, which plan 55 §10.4 records as Redox's only spelling of a block
/// expression: that is why `-> do` keeps parsing with nothing written for it
/// here, why both spellings emit the same Rust, and why leaf 7 can retire the
/// keyword without this production changing.
///
/// The inline body is one line and takes an assignment tail, which is
/// criterion 3's second half — `Exit::Success _ -> enriched += 1` was
/// `RDX0101` before this leaf, because `parse_expr` stops at `+=` and only
/// `Parser::stmt` read what follows. `assign_tail` is that reading, shared.
pub(crate) fn arm_body(p: &mut Parser) -> Result<Expr, ParseError> {
  if p.check(&TokenKind::Newline) {
    let body = p.statement_block("expected an indented `case` arm body")?;
    let span = body.span;
    return Ok(Expr::Do {
      label: None,
      body,
      span,
    });
  }

  let start = p.peek_span().0;
  let value = parse_expr(p, 0)?;
  assign_tail(p, value, start)
}
