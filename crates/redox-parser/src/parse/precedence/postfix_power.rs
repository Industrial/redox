/// Binding power of the postfix operators — call, index, field, method.
///
/// Above every binary operator, so `a.b + c` is `(a.b) + c`.
pub(crate) const POSTFIX_POWER: u8 = 27;
