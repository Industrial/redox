use std::path::PathBuf;

/// `fmt`'s resolved arguments — plan 34, leaf 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FmtArgs {
  /// One or more `.sdx` files. Plural, unlike `build`'s single input, because
  /// `redox fmt --check examples/*.sdx` is the gate plan 34 §1.5 wants and a
  /// one-file-at-a-time command would make it a shell loop whose exit status
  /// nobody checks.
  pub(crate) inputs: Vec<PathBuf>,
  /// Report and exit non-zero instead of writing.
  pub(crate) check: bool,
  pub(crate) plain: bool,
}
