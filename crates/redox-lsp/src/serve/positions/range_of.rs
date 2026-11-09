use crate::serve::positions::position_of::position_of;
use lsp_types::Range;

/// A byte span in the source, as an LSP range.
///
/// # The protocol counts UTF-16 code units, and this is where that is handled
///
/// LSP positions are `(line, character)` where `character` is an offset in
/// **UTF-16 code units** by default. Redox's spans are byte offsets. For ASCII
/// the three agree and for anything else they do not, so a server that
/// subtracted byte offsets would mis-place every diagnostic on a line
/// containing an accent — and place it correctly on every line in the test
/// corpus, which is what makes this worth doing right the first time.
///
/// `examples/strings.sdx` has non-ASCII content, so the corpus does reach
/// this.
pub(crate) fn range_of(source: &str, span: (usize, usize)) -> Range {
  Range {
    start: position_of(source, span.0),
    end: position_of(source, span.1),
  }
}
