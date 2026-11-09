/// The binding power just above `&&`'s left power of 9.
///
/// Parsing an operand at this power stops before `&&` *and* before `||`,
/// which is what makes a let-chain's operands separable. That `||` cannot
/// appear at the top level of a chain is **Rust's** restriction, not Redox's:
/// a let-chain joins with `&&` only.
pub(crate) const ABOVE_AND_AND: u8 = 10;
