use crate::expr_text::can_drop_paren::can_drop_paren;

use crate::expr_text::expr_text;

use redox_ast::Expr;

/// One side of a binary expression, parenthesised only where precedence needs
/// it.
///
/// `required` is the power this side must bind at: the operator's own power on
/// the left, one rung tighter on the right, because every binary operator in
/// Redox is left-associative. That asymmetry is what keeps `a - (b - c)` from
/// losing its parentheses while `(a - b) - c` loses them correctly.
pub(crate) fn operand(expr: &Expr, required: u8, indent: usize, source: &str) -> String {
  match expr {
    Expr::Paren { .. } => expr_text(expr, required, indent, source),
    _ if can_drop_paren(expr, required) => expr_text(expr, required, indent, source),
    _ => format!("({})", expr_text(expr, 0, indent, source)),
  }
}
