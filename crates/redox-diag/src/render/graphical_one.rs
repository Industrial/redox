/// The `miette`-shaped view of a diagnostic, kept out of the data type.
pub(crate) mod reportable;

use crate::Diagnostic;
use crate::render::graphical_one::reportable::Reportable;

/// Render one diagnostic the way `miette` draws it: boxed, coloured, with the
/// label pointing into a snippet of the real source.
///
/// `miette`'s own entry point is `Debug`, not `Display` — the graphical
/// handler is installed on the `Report`'s debug formatter — which is why this
/// is a `format!("{:?}")` rather than a `to_string`.
pub(crate) fn graphical_one(d: &Diagnostic) -> String {
  format!("{:?}", miette::Report::new(Reportable::new(d)))
}
