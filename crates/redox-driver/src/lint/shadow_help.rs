/// The help `RDX1003` carries — plan 54 leaf 3.
///
/// **Two fixes, because there are two intentions**, and the warning cannot
/// tell which one the author had. Plan 54 §1.4 keeps `=` binding on purpose,
/// so a shadow is legal and sometimes meant; this is a warning rather than an
/// error for exactly that reason, and a help naming only `:=` would read as
/// though the shadow were a mistake.
///
/// `mut` is in the first fix because leaving it out makes the advice fail.
/// `:=` on a binding written without `mut` is rustc's
/// `cannot assign twice to immutable variable` — plan 54 §10.2 made that a
/// `tests/check/` fixture — so an author who takes the suggestion literally
/// trades a silent defect for an error they then have to diagnose
/// themselves.
pub(crate) const SHADOW_HELP: &str = "to reassign instead, write `:=` and `mut` on the first binding; if the shadow is meant, \
   say so on the line above: `#[redox::allow(shadow)]`";
