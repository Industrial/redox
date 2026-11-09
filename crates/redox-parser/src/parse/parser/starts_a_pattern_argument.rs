use crate::parse::parser::Parser;
use redox_lexer::{Keyword, TokenKind};

/// Whether the cursor is on a token that can begin a sub-pattern.
///
/// An allow-list rather than a deny-list: a token that ends the construct must
/// never be read as an argument. Plan 14 removed `*` from `continues_line` for
/// the same reason, and the rule the two share is written at that set — *a
/// token that ends a construct should not continue a line.*
pub(crate) fn starts_a_pattern_argument(p: &Parser) -> bool {
  matches!(
    &p.peek().kind,
    TokenKind::Ident(_)
      | TokenKind::Int(_)
      | TokenKind::Float(_)
      | TokenKind::Char(_)
      | TokenKind::Byte(_)
      | TokenKind::Str { .. }
      | TokenKind::LParen
      | TokenKind::Kw(Keyword::True)
      | TokenKind::Kw(Keyword::False)
      | TokenKind::Kw(Keyword::SelfValue)
  )
}
