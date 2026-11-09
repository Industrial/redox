use crate::expr_text::power_of::power_of;
use redox_ast::Expr;

/// Whether the parenthesis around `inner` is redundant, given the binding
/// power the surrounding context requires.
///
/// §3 row 3 — *"parentheses that are only grouping: removed where precedence
/// allows"* — and the one place plan 34 can change what a program means. The
/// implementation is two lines because the decision has to be reviewable.
///
/// A `None` from `power_of` is a decline, and a decline keeps the parenthesis.
/// So the rule is: drop it only when the inner expression is **known** to bind
/// at least as tightly as the context demands.
pub(crate) fn can_drop_paren(inner: &Expr, required: u8) -> bool {
  power_of(inner).is_some_and(|power| power >= required)
}
