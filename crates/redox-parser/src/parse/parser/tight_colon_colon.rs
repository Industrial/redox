use crate::parse::parser::Parser;
use crate::parse::parser::spaced_colon_colon::spaced_colon_colon;
use redox_lexer::TokenKind;

/// A `::` that joins a path rather than ascribing a type.
///
/// The complement of [`spaced_colon_colon`], and deliberately expressed as
/// its negation so the two cannot drift apart: every `::` is one or the other.
pub(crate) fn tight_colon_colon(p: &Parser) -> bool {
  p.check(&TokenKind::ColonColon) && !spaced_colon_colon(p)
}
