use crate::parse::parser::Parser;
use redox_lexer::TokenKind;

/// Whether the cursor is on a **spaced** `::` — doc 18 §5.2's one surviving
/// use of adjacency lexing.
///
/// > a spaced `::` ascribes, a tight `::` is a path.
///
/// ```text
/// kind = Error::Malformed      -- tight: a Rust path, unchanged
/// total = sum :: u64           -- spaced: an ascription
/// ```
///
/// Doc 03 §21 recorded adjacency as load-bearing for three decisions — calls,
/// subtraction sections and unary minus. Juxtaposition removed the first and
/// Haskell's missing subtraction section removed the other two, and this one
/// arrived to replace all three. It is the only place in Redox where a stray
/// space changes meaning, which doc 18 §10.1 item 3 names as the thing to
/// watch.
///
/// **Both sides are checked**, not just the left. `a ::b` and `a:: b` are
/// neither a path nor an ascription, and treating them as one would make the
/// rule asymmetric for no reason. The rule as written also happens to agree
/// with both communities' formatters — Haskell always spaces `::`, Rust never
/// spaces a path — so it is a rule people already follow.
pub(crate) fn spaced_colon_colon(p: &Parser) -> bool {
  p.check(&TokenKind::ColonColon) && p.peek().ws_before && p.peek_at(1).is_some_and(|t| t.ws_before)
}
