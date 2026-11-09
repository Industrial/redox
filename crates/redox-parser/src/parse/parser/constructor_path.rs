use crate::parse::parser::Parser;
use crate::parse::parser::tight_colon_colon::tight_colon_colon;
use redox_lexer::TokenKind;

/// The `::`-joined path at the cursor if it begins uppercase, consumed.
///
/// Returns `None` without advancing when the cursor is not on an uppercase
/// identifier, so the caller can fall back to reading one pattern.
pub(crate) fn constructor_path(p: &mut Parser) -> Option<String> {
  let TokenKind::Ident(first) = &p.peek().kind else {
    return None;
  };
  if !first.chars().next().is_some_and(char::is_uppercase) {
    return None;
  }

  let mut text = first.clone();
  p.advance();
  // A **tight** `::` only, for doc 18 §5.2's reason: a spaced `::` is an
  // ascription marker, and reading it as a separator here would eat the type.
  while tight_colon_colon(p) {
    p.advance();
    match &p.peek().kind {
      TokenKind::Ident(next) => {
        text.push_str("::");
        text.push_str(next);
        p.advance();
      }
      // A malformed path is left for `pattern` to report against the token
      // that is actually wrong.
      _ => break,
    }
  }
  Some(text)
}
