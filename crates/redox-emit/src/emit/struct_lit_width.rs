/// The widest comma-joined field text a record **literal** may have and still
/// be written on one line.
///
/// `Point { x: 10, y: 20 }` stays inline; past this it is one field per line.
/// rustfmt's `struct_lit_width`, which defaults to 18 and — unlike
/// `struct_variant_width` — is measured against exactly this text, confirmed
/// by bisecting the boundary: 18 characters stays inline and 19 expands.
///
/// Pinned rather than discovered at run time, for the reasons given on
/// `STRUCT_VARIANT_WIDTH`.
pub(crate) const STRUCT_LIT_WIDTH: usize = 18;
