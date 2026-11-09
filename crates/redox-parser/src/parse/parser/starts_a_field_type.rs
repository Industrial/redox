use redox_lexer::{Keyword, TokenKind};

/// Whether a tuple constructor's next field type starts here — doc 18 §2.4.
///
/// Deliberately a small set rather than "anything that is not a terminator".
/// A wrong answer turns a unit constructor into a tuple one with a nonsense
/// field, and doc 02 §5.2's worst failure mode is output that reads as Rust
/// and means something else — so the set lists what a type can begin with and
/// nothing is admitted by omission.
pub(crate) fn starts_a_field_type(kind: &TokenKind) -> bool {
  matches!(
    kind,
    TokenKind::Ident(_)
      | TokenKind::Amp
      | TokenKind::Star
      | TokenKind::LParen
      | TokenKind::LBracket
      | TokenKind::Kw(Keyword::SelfType | Keyword::Dyn | Keyword::Impl)
  )
}
