use crate::diagnostic::Diagnostic;

/// `redox_fmt`'s result.
///
/// `changed` saves a caller a string comparison it would otherwise do on every
/// call, and makes *is this file already canonical* — which is what
/// `redox fmt --check` answers — a question this tool answers too, without a
/// second tool and without a `--check` flag whose only effect would be to
/// withhold the text.
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct FormatResponse {
  /// Whether the file could be formatted at all. False means it did not parse.
  pub ok: bool,
  /// The canonical text. `None` only when `ok` is false.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub formatted: Option<String>,
  /// Whether the canonical text differs from the file on disk.
  pub changed: bool,
  /// The parse error, when there is one. Empty otherwise — the formatter
  /// itself produces no diagnostics.
  pub diagnostics: Vec<Diagnostic>,
}
