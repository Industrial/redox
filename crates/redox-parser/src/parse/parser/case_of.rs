use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::case_arm::case_arm;
use crate::parse::parser::parse_expr::parse_expr;
use redox_ast::Expr;
use redox_diag::Code;
use redox_lexer::{Keyword, TokenKind};

/// Parse doc 18 §2.3's `case e of`, with one `->` arm per indented line.
///
/// ```text
/// case age of
///   n | n > 65      -> "senior"
///   18..=64         -> "adult"
///   Teen or PreTeen -> "young"
///   _               -> "newborn"
/// ```
///
/// The scrutinee needs no terminator token of its own: `of` is a reserved
/// word (doc 18 §7), so the Pratt loop stops at it the way it stops at any
/// keyword. That is the second thing the keyword commit bought — the first
/// being that `otherwise` cannot be shadowed.
///
/// The arm arrow is `->`, which is what frees `=>` for constraints (§7), and
/// the guard is `|` after the pattern, which is unambiguous only because
/// alternation is spelled `or` (§5.4).
pub(crate) fn case_of(p: &mut Parser) -> Result<Expr, ParseError> {
  let start = p.peek_span().0;
  p.expect(&TokenKind::Kw(Keyword::Case), "expected `case`")?;
  let scrutinee = parse_expr(p, 0)?;
  p.expect(
    &TokenKind::Kw(Keyword::Of),
    "expected `of` after the scrutinee",
  )?;
  p.expect(&TokenKind::Newline, "expected a line break after `of`")?;
  p.expect_with(
    Code::EmptyBody,
    &TokenKind::Indent,
    "expected indented `case` arms",
  )?;

  let mut arms = Vec::new();
  loop {
    p.skip_separators();
    if p.check(&TokenKind::Dedent) || p.at_eof() {
      break;
    }
    arms.push(case_arm(p)?);
  }
  p.expect(&TokenKind::Dedent, "expected the `case` arms to end")?;

  if arms.is_empty() {
    return Err(ParseError::with(
      Code::EmptyBody,
      "a `case` must have at least one arm",
      (start, p.prev_span().1),
    ));
  }

  Ok(Expr::Case {
    scrutinee: Box::new(scrutinee),
    arms,
    span: (start, p.prev_span().1),
  })
}
