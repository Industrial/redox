use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::error_variant::error_variant;
use redox_ast::ErrorsDecl;
use redox_diag::Code;
use redox_lexer::{Keyword, TokenKind, Trivia};

/// Parse doc 03 §12.5's `errors` declaration, kept by doc 18 §8.
///
/// ```text
/// errors ConfigError
///   Missing String        "missing key: #{0}"
///   Invalid String u32    "invalid value #{0} at line #{1}"
///   from io::Error as Io  "io error: #{0}"
/// ```
///
/// One indented row per variant, each ending in its `Display` message. The
/// message is what terminates a row's field list: a string literal cannot
/// begin a type, so no separator is needed between the two — which is why
/// the rows read as a table.
pub(crate) fn errors_decl(
  p: &mut Parser,
  leading: Vec<Trivia>,
  start: usize,
) -> Result<ErrorsDecl, ParseError> {
  p.expect(&TokenKind::Kw(Keyword::Errors), "expected `errors`")?;
  let name = p.ident("expected the error type's name")?;
  p.newline_with_trailing("expected a line break after the error type's name")?;
  p.expect_with(
    Code::EmptyBody,
    &TokenKind::Indent,
    "expected indented error variants",
  )?;

  let mut variants = Vec::new();
  loop {
    p.skip_separators();
    if p.check(&TokenKind::Dedent) || p.at_eof() {
      break;
    }
    variants.push(error_variant(p)?);
  }
  p.expect(&TokenKind::Dedent, "expected the error variants to end")?;

  if variants.is_empty() {
    return Err(ParseError::with(
      Code::EmptyBody,
      "an `errors` declaration must have at least one variant",
      (start, p.prev_span().1),
    ));
  }

  Ok(ErrorsDecl {
    name,
    variants,
    span: (start, p.prev_span().1),
    leading,
  })
}
