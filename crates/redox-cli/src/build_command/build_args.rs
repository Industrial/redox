use std::path::PathBuf;

/// What `redox build` was asked to do.
///
/// A struct rather than a tuple because plan 05 added a third field and a
/// `(PathBuf, PathBuf, bool)` return no longer says which bool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BuildArgs {
  pub(crate) input: PathBuf,
  pub(crate) output: PathBuf,
  /// Render diagnostics without colour, Unicode or terminal-width wrapping.
  pub(crate) plain: bool,
}
