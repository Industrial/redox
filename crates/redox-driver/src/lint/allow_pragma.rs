use crate::lint::attribute_pragma::attribute_pragma;
use crate::lint::comment_pragma::comment_pragma;
use redox_lexer::Trivia;

/// Whether one trivium is `RDX1001`'s suppression pragma, in either spelling.
///
/// ```text
/// #[redox::allow(unwrap)]                        -- plan 53 leaf 4
/// port = raw.unwrap ()
///
/// port = raw.unwrap ()  -- redox: allow unwrap   -- plan 38 §1.3, deprecated
/// ```
///
/// **Two spellings, one meaning**, for the rest of this milestone. Plan 53 §6
/// pays that cost deliberately rather than migrating in the same row: the
/// comment form is in an unknown amount of outside code, and plan 64 is where
/// the comment-carried semantics are cleaned up in one pass. The deprecation
/// is not silent — the comment form also raises `RDX1002`, which names the
/// attribute.
///
/// The argument for *moving* to an attribute is plan 53 §1.3, and it reverses
/// plan 38 §1.3. Plan 38 rejected both attribute routes together: passed
/// through, `#[allow(unwrap)]` makes rustc say `unknown lint`; consumed
/// instead, plan 18 §1.3's *attributes are trivia* rule becomes conditional
/// for a single spelling. Only the second objection survives a **consumed**
/// namespaced attribute, because nothing reaches rustc at all — and plan 53
/// pays it knowingly, having watched plan 48 find `redox fmt` delete comments
/// from a `.sdx`. Semantics in trivia a formatter may remove is the larger
/// cost.
///
/// Which spelling a trivium is matters to two callers, so each spelling has
/// its own predicate: [`attribute_pragma`] and [`comment_pragma`].
pub(crate) fn allow_pragma(trivium: &Trivia) -> bool {
  attribute_pragma(trivium) || comment_pragma(trivium)
}
