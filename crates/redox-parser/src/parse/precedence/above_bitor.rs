use crate::parse::precedence::binary_power::binary_power;
use redox_lexer::TokenKind;

/// The minimum binding power that stops a Pratt loop at a bare `|`.
///
/// Doc 18 §5.4: **"the first `|` following an equation head or a `case`
/// pattern opens the guard list"**, and inside `[ … ]` the first `|` opens a
/// comprehension's qualifier list. In all three places the bar is structural,
/// so the expression to its left must not absorb it as bitwise or.
///
/// Parsing the element at this floor leaves the bar for the caller. A genuine
/// bitwise or in an element therefore needs parentheses, which is the rule
/// §5.4 implies and the price of the bar being structural.
///
/// **Derived from the table, not written as a number**, so a change to
/// `BitOr`'s precedence cannot leave this behind — which is exactly the kind
/// of silent drift a second copy of a precedence invites.
pub(crate) fn above_bitor() -> u8 {
  let (_, left, _) = binary_power(&TokenKind::Or).expect("`|` is in the precedence table");
  left + 1
}
