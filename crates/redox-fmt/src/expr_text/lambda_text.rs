/// One parameter: a name, optionally `mut`, optionally typed.
pub(crate) mod param_text;

use crate::expr_text::expr_text;

use crate::expr_text::lambda_text::param_text::param_text;

use redox_ast::{Expr, Param};

/// A lambda — `\n -> n + 1`, doc 18 §4.
///
/// A parameter carries an optional type and a `mut` marker, both printed where
/// the parser reads them. A typed parameter is spelled `(n : u64)`: the
/// parentheses are not grouping, they are what separates one parameter's type
/// from the next parameter, and doc 18 §5.2's since-retired rule that a spaced
/// `::` ascribed
/// applies inside them.
///
/// The body is printed at power 0 and never parenthesised. A lambda body
/// extends as far right as it can, so there is nothing for a parenthesis to
/// protect it from — and `power_of` declines on the lambda itself for the same
/// reason, which is what keeps a surrounding parenthesis in place.
pub(crate) fn lambda_text(
  params: &[Param],
  body: &Expr,
  is_move: bool,
  indent: usize,
  source: &str,
) -> String {
  let mut out = String::from("\\");
  if is_move {
    out.push_str("move ");
  }
  let mut first = true;
  for param in params.iter() {
    if !first {
      out.push(' ');
    }
    first = false;
    out.push_str(&param_text(param));
  }
  // An unlabelled block body has **nothing on its own first line** — the
  // text `expr_text` returns begins with the block's first statement, already
  // padded to `indent + 1` — so the arrow takes the newline and ` -> ` in
  // front of it would be trailing whitespace. Exactly `binding_text`'s choice
  // between `"= "` and `"=\n"`, and for exactly its reason.
  //
  // A labelled block prints `'name` on the first line, so it stays inline;
  // `expr_text`'s own `Block` arm draws the same distinction.
  //
  // Found by plan 69 running `redox fmt --check` over this crate's own ported
  // source: five files map a multi-statement lambda over a collection, and
  // the formatter rewrote every one of them into source that does not parse.
  // No corpus file had a lambda with a block body before the port wrote them,
  // which is why `lambda_text` never needed the rule `binding_text` has.
  let separator = match body {
    Expr::Block { label: None, .. } => " ->\n",
    _ => " -> ",
  };
  // Piped, because a `case`-valued binding swallows a following `.name`
  // statement — `format.sdx` has the rule and the probes.
  out.push_str(separator);
  out.push_str(&expr_text(body, 0, indent, source));
  out
}
