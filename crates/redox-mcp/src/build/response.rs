use crate::diagnostic::Diagnostic;

/// `redox_build`'s result.
///
/// `output` and `map` are the paths **written**, returned rather than left to
/// be inferred — the sidecar's name is a convention (`main.xrs` has
/// `main.xrs.map`) and a convention a client has to reimplement is a
/// convention that will drift.
///
/// Both are `None` when nothing was written, which is the parse-failure case:
/// `ok` is false and the reason is in `diagnostics`.
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct BuildResponse {
  /// Whether the `.xrs` was written. False means the `.sdx` did not parse.
  pub ok: bool,
  /// The `.xrs` written, when one was.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub output: Option<String>,
  /// The `.xrs.map` sidecar written, when one was.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub map: Option<String>,
  /// Lints on a successful build; the parse error on a failed one.
  pub diagnostics: Vec<Diagnostic>,
}
