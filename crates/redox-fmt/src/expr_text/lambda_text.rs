pub(crate) mod param_text;

use crate::expr_text::expr_text;
use crate::expr_text::lambda_text::param_text::param_text;
use redox_ast::{Expr, Param};

/// A lambda — `\n -> n + 1`, doc 18 §4.
///
/// A parameter carries an optional type and a `mut` marker, both printed where
/// the parser reads them. A typed parameter is spelled `(n :: u64)`: the
/// parentheses are not grouping, they are what separates one parameter's type
/// from the next parameter, and doc 18 §5.2's rule that a spaced `::` ascribes
/// applies inside them.
///
/// The body is printed at power 0 and never parenthesised. A lambda body
/// extends as far right as it can, so there is nothing for a parenthesis to
/// protect it from — and `power_of` declines on the lambda itself for the same
/// reason, which is what keeps a surrounding parenthesis in place.
pub(crate) fn lambda_text(params: &[Param], body: &Expr, is_move: bool, source: &str) -> String {
  let mut out = String::from("\\");
  if is_move {
    out.push_str("move ");
  }
  for (i, param) in params.iter().enumerate() {
    if i > 0 {
      out.push(' ');
    }
    out.push_str(&param_text(param));
  }
  out.push_str(" -> ");
  out.push_str(&expr_text(body, 0, source));
  out
}
