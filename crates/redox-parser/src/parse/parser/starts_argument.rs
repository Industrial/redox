use redox_lexer::{Keyword, TokenKind};

/// Whether `kind` can begin a parenless argument without being mistaken for
/// an infix operator continuing the callee.
///
/// A sibling of `stmt_call` rather than a child, because doc 06 §2.4 caps
/// depth at three levels below `src/`.
///
/// `-`, `!`, `&` and `*` are excluded on purpose: `a - b` as a statement would
/// otherwise read as `a(-b)`, and telling those apart is exactly what plan
/// 19's whitespace-adjacency rule is for.
pub(crate) fn starts_argument(kind: &TokenKind) -> bool {
  matches!(
    kind,
    TokenKind::Int(_)
      | TokenKind::Float(_)
      | TokenKind::Str { .. }
      | TokenKind::RawStr { .. }
      | TokenKind::ByteStr(_)
      | TokenKind::CStr(_)
      | TokenKind::Char(_)
      | TokenKind::Byte(_)
      | TokenKind::Ident(_)
      | TokenKind::LParen
      | TokenKind::At
      | TokenKind::AtAt
      | TokenKind::Kw(Keyword::True)
      | TokenKind::Kw(Keyword::False)
  )
}
