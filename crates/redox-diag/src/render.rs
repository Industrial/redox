pub(crate) mod anchor_span;
pub(crate) mod caret_width;
pub(crate) mod graphical_one;
pub(crate) mod line_col;
pub(crate) mod plain_one;
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
