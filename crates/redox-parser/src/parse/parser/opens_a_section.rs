use redox_lexer::TokenKind;

/// Whether a token may open a **right** section — doc 18 §4.
///
/// Every binary operator may, **except the four that are also prefix
/// operators**: `-`, `&`, `*` and `!`. A leading one of those is the prefix
/// reading, always.
///
/// Doc 18 §4 states the case for `-` outright — `(- 1)` is negation, which is
/// what retires doc 03 decision 50's adjacency rule — and the same argument
/// settles the rest. `&` is the one that proved it: §4 itself notes that
/// Haskell's reverse-application `&` "is Rust's reference sigil", and
/// `(&ConfigError::Missing x)` was read as a section of bitwise and until
/// this existed, turning a reference into a closure.
///
/// `*` and `!` are here by the same rule rather than by a separate finding,
/// which is the point of expressing it as a rule: `(*p)` is a deref and
/// `(!flag)` a negation, and neither needs its own discovery.
pub(crate) fn opens_a_section(kind: &TokenKind) -> bool {
  !matches!(
    kind,
    TokenKind::Minus | TokenKind::Amp | TokenKind::Star | TokenKind::Bang
  )
}
