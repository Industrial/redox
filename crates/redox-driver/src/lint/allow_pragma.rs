use crate::lint::attribute_pragma::attribute_pragma;
use crate::lint::comment_pragma::comment_pragma;
use redox_diag::Lint;
use redox_lexer::Trivia;

/// Whether one trivium suppresses `lint`, in either spelling.
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
///
/// # Why `lint` is a parameter
///
/// Plan 53 wrote `unwrap` into both spellings, because there was one lint to
/// write. Plan 54 §10.6 names the consequence it would have had: a second
/// hard-coded name is a second pair of strings in two crates, and an author's
/// intentional shadow would have warned forever. So the name is a parameter,
/// and a pragma silences the lint it names and no other —
/// `#[redox::allow(unwrap)]` above a shadow is not a suppression, which is
/// right: a suppression is a claim about *one* warning.
pub(crate) fn allow_pragma(trivium: &Trivia, lint: Lint) -> bool {
  attribute_pragma(trivium) == Some(lint) || comment_pragma(trivium) == Some(lint)
}
