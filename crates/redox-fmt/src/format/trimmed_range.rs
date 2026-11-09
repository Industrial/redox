/// The byte range of an item's actual text inside the region it owns — plan
/// 34, leaf 1.
///
/// A region runs from one item's start to the next's, and its ends are padded
/// with whitespace that belongs to neither: a declaration's span reaches the
/// newline that ends it, and a comment trivium's span starts at the newline
/// *before* the comment. So the blank line between two items is split across
/// the boundary, with part of it at the end of one region and part at the
/// start of the next.
///
/// That is why the gap cannot be measured from either region alone — the first
/// attempt counted trailing blank lines of the earlier region and saw none,
/// because the blank line had landed in the later one. Trimming both to their
/// real text and measuring the source *between* them is the model that works.
pub(crate) fn trimmed_range(source: &str, start: usize, end: usize) -> Option<(usize, usize)> {
  let start = start.min(source.len());
  let end = end.min(source.len());
  let slice = &source[start..end];
  let first = slice.find(|c: char| !c.is_whitespace())?;
  let last = slice.rfind(|c: char| !c.is_whitespace())?;
  // `last` indexes the final non-whitespace byte, so the range ends after it.
  Some((start + first, start + last + 1))
}
