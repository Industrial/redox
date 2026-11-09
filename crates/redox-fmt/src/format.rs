pub(crate) mod blank_line_in;
pub(crate) mod item_start;
pub(crate) mod item_text;
pub(crate) mod trimmed_range;

use crate::format::blank_line_in::blank_line_in;
use crate::format::item_start::item_start;
use crate::format::item_text::item_text;
use crate::format::trimmed_range::trimmed_range;
use redox_ast::Program;

/// Format a parsed program back into canonical Redox — plan 34, leaf 1.
///
/// # What this increment does, and what it deliberately does not
///
/// It canonicalises **file-level structure**: a run of blank lines between
/// items collapsed to one, no leading blank lines, exactly one trailing
/// newline, and no trailing whitespace on any line. Every item's own text is
/// passed through **verbatim**, sliced out of the source.
///
/// A gap of *zero* blank lines stays zero, which is not a detail:
/// `blank_line_in` records why an author's grouping of adjacent imports
/// decides what rustfmt is allowed to do to them.
///
/// That is plan 34's criterion 7 — *"a construct the formatter does not yet
/// print is passed through verbatim rather than mangled"* — used as the
/// starting point rather than as a fallback. The consequence is worth stating:
/// this formatter **cannot lose text**, cannot reorder anything, and cannot
/// change what a program means, because it does not rewrite any construct. It
/// is also correspondingly modest.
///
/// §1.2 of the plan says the row will be `partial` for a while. Each later
/// leaf replaces one span-slice with a real printer, and the three property
/// tests of §1.4 hold at every step by construction — which is the reason to
/// build it in this order.
///
/// # Items are cut at boundaries, then trimmed
///
/// An item owns the source from its own start to the **next item's** start,
/// which is not the same as its span: a declaration's span runs to the newline
/// that ends it, and can reach *into the following comment*, which is also the
/// next item's leading trivia. Slicing by spans emitted that comment **twice**
/// — caught in `examples/case.sdx` by the property test that counts `--`
/// markers.
///
/// Within a region, `trimmed_range` finds the real text, and the whitespace
/// left over at each end is the gap `blank_line_in` measures.
pub fn format(program: &Program, source: &str) -> String {
  let starts: Vec<usize> = program.items.iter().map(item_start).collect();

  // Where the last item ends: before a trailing comment if there is one, and
  // at the end of the source otherwise.
  let tail = program
    .trailing
    .first()
    .map_or(source.len(), |trivium| trivium.span.0);

  // Each item's real text range, in order, plus the file's trailing trivia as
  // one more block — so the gap logic treats it like any other neighbour.
  let mut blocks: Vec<(usize, usize)> = Vec::new();
  for (i, _) in program.items.iter().enumerate() {
    let end = starts.get(i + 1).copied().unwrap_or(tail);
    if let Some(range) = trimmed_range(source, starts[i], end) {
      blocks.push(range);
    }
  }
  if let Some(range) = trimmed_range(source, tail, source.len()) {
    blocks.push(range);
  }

  let mut out = String::new();
  for (i, range) in blocks.iter().enumerate() {
    out.push_str(&item_text(source, *range));
    out.push('\n');
    if let Some(next) = blocks.get(i + 1)
      && blank_line_in(&source[range.1..next.0])
    {
      out.push('\n');
    }
  }
  out
}
