pub(crate) mod allow_pragma;
pub(crate) mod line_of;
pub(crate) mod unwrap_call_sites;
pub(crate) mod unwrap_message;

use crate::lint::unwrap_call_sites::unwrap_call_sites;
use crate::lint::unwrap_message::UNWRAP_MESSAGE;
use redox_diag::{Code, Diagnostic, Severity};
use std::path::Path;

/// Every lint, as warnings — doc 03 §12.7, plan 38.
///
/// One lint today: `RDX1001`, `unwrap` discouragement. `RDX1000`+ is a band
/// and this opens it with one code; the next lint takes the second number and
/// this function's shape.
///
/// # Why the driver and not the parser
///
/// Plan 30's `check_callback_tails` is a whole-program pass in the **parser**,
/// and it produces `ParseError`s — which become hard errors, because
/// `ParseFailure` has no notion of a warning. A lint cannot go that way.
///
/// `Diagnostic` is the type that carries a [`Severity`], and the driver is
/// where `Diagnostic`s are built (`diagnose`). So a lint runs here, over an
/// AST that already parsed, and returns warnings the caller renders and does
/// not fail on.
///
/// That also means a lint never runs on a file that failed to parse, which is
/// right: a warning about a program nobody can compile is noise — and it is
/// why this takes the source rather than the AST: the one lint it carries is
/// lexical, for the reason `unwrap_call_sites` records.
pub fn lint(path: &Path, source: &str) -> Vec<Diagnostic> {
  let path = path.display().to_string();
  unwrap_call_sites(source)
    .into_iter()
    .map(|span| Diagnostic {
      code: Code::UnwrapUsed.into(),
      severity: Severity::Warning,
      message: UNWRAP_MESSAGE.to_string(),
      span,
      help: Some("if a panic is intended here, say so: `-- redox: allow unwrap`".to_string()),
      path: path.clone(),
      source: source.to_string(),
    })
    .collect()
}
