use crate::line_col::line_col;
use redox_parser::ParseFailure;
use std::fmt::Write as _;
use std::path::Path;

/// Render a failure as `error: …` plus a `--> file:line:col` location.
///
/// `miette` rendering is plan 05; the spans it needs are already carried by
/// every error type.
pub(crate) fn render(path: &Path, source: &str, failure: &ParseFailure) -> String {
  let mut out = String::new();
  match failure {
    ParseFailure::Lex(e) => {
      let _ = writeln!(out, "error: {}", e.message);
      let _ = writeln!(out, "  --> {}:{}:{}", path.display(), e.line, e.col);
    }
    ParseFailure::Parse(errors) => {
      for e in errors {
        let (line, col) = line_col(source, e.span.0);
        let _ = writeln!(out, "error: {}", e.message);
        let _ = writeln!(out, "  --> {}:{line}:{col}", path.display());
      }
    }
  }
  out.pop();
  out
}
