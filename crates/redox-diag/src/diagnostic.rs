pub(crate) mod code;

use crate::Code;
use crate::Span;

/// One diagnostic, ready to render.
///
/// Plan 05 (`docs/10 - 05 - Diagnostics.md`) leaf 2. Shaped after Emerald's
/// `SemaError` (`crates/emerald-cli/src/main.rs:47-81`), which supplies
/// `source_code` and one `LabeledSpan` — the one piece of its diagnostics work
/// that transfers directly.
///
/// `miette` renders this; the stage crates never see it. An error carries a
/// span and a message, and only the CLI renders (doc 06 §6 rule 2), which
/// keeps the renderer swappable.
#[derive(Debug, Clone)]
pub struct Diagnostic {
  pub code: Code,
  pub message: String,
  pub span: Span,
  /// A one-line suggestion, shown after the snippet.
  pub help: Option<String>,
  /// The file name, for the location line.
  pub path: String,
  /// The whole source, so the snippet has context.
  pub source: String,
}

impl std::fmt::Display for Diagnostic {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl std::error::Error for Diagnostic {}
