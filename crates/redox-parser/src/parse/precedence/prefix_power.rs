/// Binding power of the prefix operators — `-`, `!`, `&`, `*`.
///
/// Between the arithmetic and postfix levels, matching Rust: `-a * b` is
/// `(-a) * b`, and `!a && b` is `(!a) && b`.
pub(crate) const PREFIX_POWER: u8 = 25;
