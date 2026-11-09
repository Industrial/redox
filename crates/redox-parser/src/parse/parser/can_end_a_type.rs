use redox_lexer::{Keyword, TokenKind};

/// Whether a token can be the **last** token of a type.
///
/// Used to find the boundary between two juxtaposed payload types: a
/// whitespace gap after something that can end a type, before something that
/// can begin one, is where `Invalid String u32` splits into two fields.
///
/// The set is what it excludes that matters. `&`, `*`, `mut`, `dyn`, `impl`
/// and a lifetime all *demand* a type after them, so a gap following one of
/// those is inside a type rather than between two — which is what keeps
/// `&'static str` and `&mut Self` whole.
pub(crate) fn can_end_a_type(kind: &TokenKind) -> bool {
  matches!(
    kind,
    TokenKind::Ident(_)
      | TokenKind::Gt
      | TokenKind::Shr
      | TokenKind::RParen
      | TokenKind::RBracket
      | TokenKind::Kw(Keyword::SelfType)
  )
}
