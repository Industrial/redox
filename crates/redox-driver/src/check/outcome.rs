use redox_diag::Diagnostic;

/// What [`crate::check`] found.
///
/// Plan 50 (`docs/44 - 50 - MCP Server.md`) leaf 1. Two fields rather than one,
/// because *are there diagnostics* and *did the check fail* are different
/// questions and conflating them was already a bug waiting in
/// `check_command`: rustc's warnings are diagnostics and do not fail a check,
/// which is rustc's own behaviour and not something Redox should second-guess.
#[derive(Debug, Clone, Default)]
pub struct CheckOutcome {
  /// Redox's lints, then rustc's diagnostics remapped onto the `.sdx`, in that
  /// order — so a reader sees Redox's own warnings before rustc's.
  pub diagnostics: Vec<Diagnostic>,
  /// Whether anything in here stops the build.
  ///
  /// Not `diagnostics.is_empty()`: a check with only warnings passes.
  pub failed: bool,
}
