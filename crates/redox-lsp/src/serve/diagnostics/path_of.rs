use lsp_types::Uri;
use std::path::Path;

/// A filesystem path for a `file:` URI, for the driver's `path` argument.
///
/// Only used in the diagnostic's own `path` field, which the LSP reply does
/// not carry — the editor already knows which file it asked about. So a URI
/// this cannot interpret falls back to its own text rather than failing: the
/// path is for a renderer that is not running here.
pub(crate) fn path_of(uri: &Uri) -> std::path::PathBuf {
  let text = uri.as_str();
  Path::new(text.strip_prefix("file://").unwrap_or(text)).to_path_buf()
}
