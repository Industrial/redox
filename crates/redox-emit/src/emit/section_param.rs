/// The parameter name a section's closure binds — doc 18 §4.
///
/// `__rdx_1`, which is the generated-identifier prefix doc 02 reserves, so a
/// section can never capture or shadow a name the author wrote. Doc 18 §4's
/// own table spells it, and it is a constant here rather than a literal in
/// five format strings.
pub(crate) const SECTION_PARAM: &str = "__rdx_1";
