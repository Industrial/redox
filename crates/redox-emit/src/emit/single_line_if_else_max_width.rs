/// rustfmt's `single_line_if_else_max_width` default.
///
/// A two-armed `if … { … } else { … }` whose whole text is no wider than this
/// is written on one line. Measured on the expression, like every other width
/// rule the emitter honours, and confirmed at the boundary: 50 characters is
/// joined and 51 is broken.
///
/// Only a **two-armed** chain qualifies. An `else if` in the middle disowns
/// the rule, and an `if` with no `else` is never joined — which is why doc 18
/// §2.3's three-row `tier` ladder was unaffected and the two-row `halve`
/// ladder in `examples/testing.sdx` was the one that found this.
///
/// Pinned rather than discovered at run time, for the reasons given on
/// `STRUCT_VARIANT_WIDTH`.
pub(crate) const SINGLE_LINE_IF_ELSE_MAX_WIDTH: usize = 50;
