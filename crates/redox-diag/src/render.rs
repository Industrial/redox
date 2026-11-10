/// Move a span onto a character that can actually be drawn under.
pub(crate) mod anchor_span;
/// How many `^` to draw under a span.
pub(crate) mod caret_width;
/// The boxed, coloured rendering, by way of `miette`.
pub(crate) mod graphical_one;
/// Resolve a byte offset to a 1-based line and column.
pub(crate) mod line_col;
/// The rendering with no colour and no width dependence.
pub(crate) mod plain_one;
/// Which source line to show, and where in it to point.
pub(crate) mod snippet_line;

use crate::Diagnostic;
use crate::render::graphical_one::graphical_one;
use crate::render::plain_one::plain_one;

/// Render every diagnostic, graphically or plainly.
///
/// Plan 05 (`docs/10 - 05 - Diagnostics.md`) leaf 4. One report per
/// diagnostic, which is Emerald's `report_driver_error` loop — plan 03's
/// two-level recovery already produces several errors and nothing yet
/// displayed more than the first.
///
/// `plain` exists for the UI tests, not for users: a corpus comparing bytes
/// cannot depend on terminal width or colour detection. Making it a flag
/// rather than an environment sniff keeps the test honest about what it
/// asserts.
pub fn render(diagnostics: &[Diagnostic], plain: bool) -> String {
  let mut out = String::new();
  for (i, d) in diagnostics.iter().enumerate() {
    if i > 0 {
      out.push('\n');
    }
    if plain {
      out.push_str(&plain_one(d));
    } else {
      out.push_str(&graphical_one(d));
    }
  }
  out
}
