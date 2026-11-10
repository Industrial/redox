use crate::diagnostic::Diagnostic;

/// `redox_check`'s result.
///
/// `ok` is **not** `diagnostics.is_empty()`. A warning is a diagnostic and does
/// not fail a check, which is rustc's own behaviour and not something Redox
/// should second-guess; Redox's lints arrive by the same route, since doc 03
/// §12.7 makes `unwrap` *"lintable rather than prohibited"*.
///
/// Two fields rather than one for that reason, mirroring
/// [`redox_driver::CheckOutcome`] — a reader asking *did it pass* and a reader
/// asking *what should I fix* want different answers.
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct CheckResponse {
  /// Whether the program compiles. False only when something fatal is in
  /// `diagnostics`.
  pub ok: bool,
  /// Redox's lints first, then rustc's diagnostics remapped onto the `.sdx`.
  pub diagnostics: Vec<Diagnostic>,
}
