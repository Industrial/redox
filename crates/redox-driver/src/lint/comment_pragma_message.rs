/// The message `RDX1002` carries — plan 53 leaf 5.
///
/// It names **the spelling to write**, because that is the whole of what this
/// diagnostic is for: the comment pragma still suppresses, so an author who
/// is told only that it is deprecated has been given no work they can do.
///
/// Beside the message, [`UNWRAP_MESSAGE`](super::unwrap_message::UNWRAP_MESSAGE)
/// for the same reason — the wording is the feature, so it lives in a named
/// constant rather than inline at the construction site.
pub(crate) const COMMENT_PRAGMA_MESSAGE: &str = "`-- redox: allow unwrap` is deprecated — write `#[redox::allow(unwrap)]` on the line above \
   instead (plan 53 §1.3)";
