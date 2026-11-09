use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::token_text::token_text;
use redox_ast::Visibility;
use redox_lexer::{Keyword, TokenKind};

/// Parse doc 03 §9.6's visibility marker, if one is here.
///
/// ```text
/// export                 pub
/// export(crate)          pub(crate)
/// export(super)          pub(super)
/// export(in path::to)    pub(in path::to)
/// (absent)               private
/// ```
///
/// Absent means private, so this returns [`Visibility::Private`] rather than
/// an error when there is no marker — a caller can ask unconditionally, and
/// every declaration does.
///
/// The `in` path is captured as **verbatim token text**, as every path in
/// Redox is (doc 03 §13.1). `in` and `crate` are reserved words and `super`
/// is not, which is why the three arms are matched differently; the shape is
/// the lexer's and not a choice made here.
pub(crate) fn visibility(p: &mut Parser) -> Result<Visibility, ParseError> {
  if !p.check(&TokenKind::Kw(Keyword::Export)) {
    return Ok(Visibility::Private);
  }
  p.advance();

  if !p.check(&TokenKind::LParen) {
    return Ok(Visibility::Public);
  }
  p.advance();

  let scope = if p.check(&TokenKind::Kw(Keyword::In)) {
    p.advance();
    let mut path = String::new();
    while !p.check(&TokenKind::RParen) && !p.at_eof() {
      path.push_str(&token_text(&p.peek().kind));
      p.advance();
    }
    if path.is_empty() {
      return Err(ParseError::expected(
        "expected a module path after `in`",
        p.peek_span(),
      ));
    }
    Visibility::In(path)
  } else {
    let word = token_text(&p.peek().kind);
    p.advance();
    match word.as_str() {
      "crate" => Visibility::Crate,
      "super" => Visibility::Super,
      _ => {
        return Err(ParseError::expected(
          "expected `crate`, `super` or `in` inside `export(…)`",
          p.prev_span(),
        ));
      }
    }
  };

  p.expect(&TokenKind::RParen, "expected `)` to close `export(…)`")?;
  Ok(scope)
}
