/// Write doc 18 §4's closure: `|params| body`.
///
/// **One lowering, two constructs.** Plan 56 §5 criterion 6 asks that an
/// operator section and a function partial application differ in the table
/// consulted and nowhere else, so both go through here: `emit_section` builds
/// a body out of an operator and `emit_application` builds one out of a
/// callee, and the closure they end up with has the same shape.
///
/// **No `move`, and that is a decision rather than an omission.** §1.3's table
/// wrote `move |__rdx_1| add(1, __rdx_1)`; §11.2 measured that sections carry
/// no capture annotation at all and have since plan 29, so the two halves of
/// criterion 6 could not both hold. Sections are the side that does not move:
/// `move` on a closure capturing a borrowed argument turns a borrow error into
/// a *move* error — a worse message about the same mistake — and §3 case 3
/// wants rustc's borrow error specifically. Sections also already have a
/// golden and a benchmark pair pinning their text, so changing them would
/// re-bless two corpora to make a new construct match an old one.
pub(crate) fn closure(params: &[String], body: &str) -> String {
  format!("|{}| {body}", params.join(", "))
}
