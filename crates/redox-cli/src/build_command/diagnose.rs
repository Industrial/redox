use redox_diag::Diagnostic;
use redox_parser::ParseFailure;
use std::path::Path;

/// Turn a [`ParseFailure`] into the diagnostics that represent it.
///
/// Plan 05 (`docs/10 - 05 - Diagnostics.md`) leaf 4. The conversion lives here
/// rather than in `redox-diag` because it mentions `redox-parser`, and
/// `redox-diag` is the lowest crate in the layering (doc 06 §3): it may not
/// depend on a stage. The CLI sits above both, so it is the right place for
/// the join.
///
/// Every stage error already carries its own code and span, so this function
/// decides nothing — it only attaches the file name and the source text that
/// rendering needs, and copies the rest across.
pub(crate) fn diagnose(path: &Path, source: &str, failure: &ParseFailure) -> Vec<Diagnostic> {
  let path = path.display().to_string();
  match failure {
    ParseFailure::Lex(e) => vec![Diagnostic {
      code: e.code,
      message: e.message.clone(),
      span: e.span,
      help: e.help.clone(),
      path,
      source: source.to_string(),
    }],
    ParseFailure::Parse(errors) => errors
      .iter()
      .map(|e| Diagnostic {
        code: e.code,
        message: e.message.clone(),
        span: e.span,
        help: None,
        path: path.clone(),
        source: source.to_string(),
      })
      .collect(),
  }
}
