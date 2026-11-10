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
///
/// **`self` and `Self` were missing, and plan 68 found it.** `.len self.xs`
/// stopped the application after `.len` and left `self.xs` a stray expression
/// — `RDX0101`, *"expected a line break after the body"*. Neither can be an
/// infix operator, so there was no ambiguity to protect against: it was an
/// omission rather than a decision, and doc 18 §4.3 is the argument against
/// it. Dropping `@name` for `self.name` made `self` an ordinary expression,
/// and an ordinary expression is an ordinary argument.
///
/// The corpus could not catch it. `examples/receiver.sdx` writes `self` eight
/// times and every one is a whole body or an operand — `current self =
/// self.value`, `plus self n = self.value + n` — and not one is a
/// **juxtaposed argument**, which is the only position this predicate
/// governs. A crate full of methods hits it on the first one.
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
      | TokenKind::Kw(Keyword::SelfValue)
      | TokenKind::Kw(Keyword::SelfType)
  )
}
