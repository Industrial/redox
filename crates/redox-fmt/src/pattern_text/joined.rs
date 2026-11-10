use crate::pattern_text::pattern_text;

use redox_ast::Pattern;

/// A comma-separated run of patterns, with no trailing comma.
///
/// §3 row 12 is free here for the reason it is free for expressions: the
/// printer works from the tree, and the tree has no record of a trailing
/// comma. The one comma that survives is the one that makes `(a,)` a tuple,
/// and that is written at the call site rather than hidden in here.
pub(crate) fn joined(items: &[Pattern]) -> String {
  // `pattern_text` passed straight to `.map`, with no lambda: it is a **free
  // function**, and plan 68's `(.name)` problem was specific to a *method*
  // reference, where `(.name)` is the field selector.
  let parts: Vec<String> = items.iter().map(pattern_text).collect();
  parts.join(", ")
}
