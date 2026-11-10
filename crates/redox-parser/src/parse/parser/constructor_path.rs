use crate::parse::parser::Parser;
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
  // `::` is a path separator and nothing else — plan 52 retired doc 18 §5.2's
  // spaced-versus-tight rule when `:` became the type marker.
  while p.check(&TokenKind::ColonColon) {
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
