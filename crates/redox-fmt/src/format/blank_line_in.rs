/// Whether a blank line separated two items in the source — plan 34, leaf 1.
///
/// `gap` is the source between the end of one item's text and the start of the
/// next's. Two or more newlines in it means at least one blank line.
///
/// **A blank line between items is significant**, which is why this exists at
/// all. Plan 14 §10.2: rustfmt's `reorder_imports` sorts within a *contiguous*
/// block, so
///
/// ```text
/// import core::task::Context
/// import core::task::Poll
/// import core::task::Waker
/// ```
///
/// is one block rustfmt may sort, and separating the three makes each its own
/// block that it may not. The author's grouping decides what the generated
/// Rust is allowed to do to their imports.
///
/// So the rule is the ordinary one: **collapse a run of blank lines to one,
/// and leave zero as zero.** The first version of this formatter had a
/// `SEPARATOR` constant and a doc comment arguing there was no exception.
/// There was, and `examples/async.sdx`'s three adjacent imports found it.
pub(crate) fn blank_line_in(gap: &str) -> bool {
  gap.matches('\n').count() > 1
}
