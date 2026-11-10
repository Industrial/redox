use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::TestDecl;
use redox_diag::Code;
use redox_lexer::{Keyword, TokenKind, Trivia};

/// Parse doc 03 §15's `test` block, kept by doc 18 §8.
///
/// ```text
/// test "adds two numbers"
///   assert! \-> add 1 2 == 3
/// ```
///
/// A description and an indented body, which is the same body shape a
/// function's clause has — so a test may contain anything a function may,
/// including the draw arrows and `case` expressions that need layout of
/// their own.
pub(crate) fn test_decl(
  p: &mut Parser,
  leading: Vec<Trivia>,
  start: usize,
) -> Result<TestDecl, ParseError> {
  p.expect(&TokenKind::Kw(Keyword::Test), "expected `test`")?;

  let description = match &p.peek().kind {
    TokenKind::Str { raw, .. } => raw.clone(),
    _ => {
      return Err(ParseError::expected(
        "expected the test's description, as a string literal",
        p.peek_span(),
      ));
    }
  };
  p.advance();
  p.newline_with_trailing("expected a line break after the description")?;

  let body = p.indented_block(Code::EmptyBody, "expected an indented test body")?;
  Ok(TestDecl {
    description,
    body,
    span: (start, p.prev_span().1),
    leading,
  })
}
