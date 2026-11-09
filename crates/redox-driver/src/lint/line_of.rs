/// The 1-based line a byte offset falls on.
///
/// Plan 38's suppression is per line, so the pragma and the call site are
/// compared by this. Counting newlines rather than keeping a line table
/// because the lint runs once per file over a handful of offsets, and a table
/// would be a second representation of the source to keep correct.
pub(crate) fn line_of(source: &str, offset: usize) -> usize {
  source[..offset.min(source.len())].matches('\n').count() + 1
}
