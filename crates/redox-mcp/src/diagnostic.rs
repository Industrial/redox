pub(crate) mod from_redox;
pub(crate) mod line_and_column;

/// One diagnostic, as an MCP client receives it.
///
/// Plan 50 (`docs/44 - 50 - MCP Server.md`) §1.3. This is a **projection** of
/// [`redox_diag::Diagnostic`] and deliberately not a `Serialize` of it, for two
/// reasons that are about the wire and not about taste:
///
/// - `Diagnostic::source` is the **whole file**. Serialised, every response
///   would carry the complete contents of every file checked, to a client that
///   already has them open.
/// - `Diagnostic::span` is **byte offsets**. Right for `miette`, useless to a
///   reader, which quotes a line and a column.
///
/// It is also the right division under doc 06 §6 rule 2 — *only the CLI
/// renders*. An MCP tool is a second renderer, and its shape is its own
/// business rather than the diagnostic type's.
///
/// # The unit
///
/// `line` and `column` are **1-based characters**. Not bytes, and not UTF-16
/// code units: `redox-lsp` has `position_of` and this does not use it, because
/// its `character` is a UTF-16 offset that the protocol demands and a reader
/// does not count. A unit away from right is worse than not reusing it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub struct Diagnostic {
  /// Redox's own code (`RDX0001`) or another compiler's passed through
  /// (`E0308`). Never renumbered — a reader who searches for a foreign code
  /// must find that compiler's documentation (plan 05).
  pub code: String,
  /// `error` or `warning`. A warning does not fail a check.
  pub severity: String,
  pub message: String,
  /// The `.sdx` this is reported against, which is the file the user wrote.
  pub path: String,
  /// 1-based line in the `.sdx`.
  pub line: usize,
  /// 1-based column in characters.
  pub column: usize,
  /// The one-line suggestion, when there is one.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub help: Option<String>,
}
