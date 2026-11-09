use crate::Diagnostic;
use crate::render::caret_width::caret_width;
use crate::render::line_col::line_col;
use crate::render::snippet_line::snippet_line;
use std::fmt::Write as _;

/// Render one diagnostic without colour or width dependence.
///
/// Deliberately close to doc 01 §29's own example output, so the plain form is
/// recognisable rather than a second dialect:
///
/// ```text
/// error[RDX0101]: expected `->` after the signature
///   --> bad_arrow.sdx:1:15
///   |
/// 1 | add = (a: i32)
///   |               ^
/// ```
///
/// The first word is the **severity**, not a constant: plan 38's `RDX1001` is
/// a warning, and this renderer said `error[RDX1001]` until it stopped being
/// one. A hardcoded word is the kind of thing that stays right for thirty-odd
/// codes and is wrong for the thirty-first.
pub(crate) fn plain_one(d: &Diagnostic) -> String {
  let (line, col) = line_col(&d.source, d.span.0);
  let level = match d.severity {
    crate::Severity::Error => "error",
    crate::Severity::Warning => "warning",
  };
  let mut out = format!("{level}[{}]: {}\n", d.code.text(), d.message);
  let _ = writeln!(out, "  --> {}:{line}:{col}", d.path);

  if let Some((line, text, col)) = snippet_line(&d.source, line, col) {
    let gutter = " ".repeat(line.to_string().len());
    let _ = writeln!(out, "{gutter} |");
    // A literal tab in the snippet would put the caret line out of step with
    // the source line, which matters most for the one diagnostic that is
    // *about* a tab. Expanding to a single space keeps the two aligned.
    let _ = writeln!(out, "{line} | {}", text.replace('\t', " "));
    let _ = writeln!(
      out,
      "{gutter} | {}{}",
      " ".repeat(col - 1),
      "^".repeat(caret_width(&d.source, d.span))
    );
  }
  if let Some(help) = &d.help {
    let _ = writeln!(out, "  = help: {help}");
  }
  out
}
