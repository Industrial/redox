use crate::diagnostic::Diagnostic;
use crate::diagnostic::line_and_column::diagnostic_line_and_column;
use redox_diag::Severity;

/// Project a compiler diagnostic onto the wire shape.
///
/// Plan 50 §1.3. The two fields that do **not** cross are the point of the
/// function: `source` is the whole file and stays behind, and `span` becomes a
/// line and a column.
///
/// The span's **start** is what is reported. A range would be more
/// information, and plan 05 keeps one so `miette` can underline a statement —
/// but an MCP client quotes a position, and offering an end column that is
/// sometimes the whole statement and sometimes one token would be a field
/// nobody could rely on. Narrowed deliberately; plan 50 §10 records it.
pub(crate) fn diagnostic_from_redox(diagnostic: &redox_diag::Diagnostic) -> Diagnostic {
  let (line, column) = diagnostic_line_and_column(&diagnostic.source, diagnostic.span.0);
  Diagnostic {
    code: diagnostic.code.text(),
    severity: match diagnostic.severity {
      Severity::Error => "error",
      Severity::Warning => "warning",
    }
    .to_string(),
    message: diagnostic.message.clone(),
    path: diagnostic.path.clone(),
    line,
    column,
    help: diagnostic.help.clone(),
  }
}
