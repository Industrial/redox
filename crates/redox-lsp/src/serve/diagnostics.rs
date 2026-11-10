/// One Redox diagnostic as the protocol's.
pub(crate) mod convert;
/// A filesystem path for a `file:` URI.
pub(crate) mod path_of;

use crate::serve::diagnostics::convert::convert;
use crate::serve::diagnostics::path_of::path_of;
use lsp_types::{Diagnostic, Uri};

/// Every diagnostic Redox itself can report for one document.
///
/// Two sources, both already public and both plan 40 §1.1's point that the
/// server contributes no analysis of its own:
///
/// - `redox_driver::diagnose` for a parse or lex failure, which is the one the
///   user sees while typing;
/// - `redox_driver::lint` for the `RDX1000+` band — plan 38's `RDX1001`, with
///   its `-- redox: allow unwrap` pragma already honoured, because the pragma
///   is `lint`'s business and not the protocol's.
///
/// # No rustc pass
///
/// Plan 40 §6 and decision 2. `redox check` shells out to rustc, which takes
/// seconds; running it per keystroke is wrong and running it on save needs a
/// cancellation and debounce design larger than the rest of the row. The
/// remap moved to the driver in leaf 1 anyway, because the move is the part
/// that rots if deferred.
///
/// # A document that parses still gets linted
///
/// The order matters and is easy to get backwards: a lint is a token scan, so
/// it works on a file that does not parse, and a parse failure should not
/// suppress it. Both lists are always computed and concatenated.
///
/// That is also why the parse result is **kept** rather than only tested.
/// Since plan 53 a lint reads suppression ranges off the tree, and the tree
/// is what a document mid-edit does not have — so it is passed when there is
/// one and `None` when there is not, which keeps the lint running on a broken
/// document at the cost of a pragma not reaching past its own line until the
/// file parses again.
pub(crate) fn diagnostics(uri: &Uri, source: &str) -> Vec<Diagnostic> {
  let path = path_of(uri);
  let mut found: Vec<Diagnostic> = Vec::new();

  let parsed = match redox_parser::parse_source(source) {
    Ok(program) => Some(program),
    Err(failure) => {
      for diagnostic in redox_driver::diagnose(&path, source, &failure) {
        found.push(convert(&diagnostic, source));
      }
      None
    }
  };
  for diagnostic in redox_driver::lint(&path, source, parsed.as_ref()) {
    found.push(convert(&diagnostic, source));
  }
  found
}
