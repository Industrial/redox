use crate::expr_text::expr_text;

use redox_ast::Expr;

/// An application the author spelled with doc 18 §2.1's `$`.
///
/// `$` is low-precedence right-associative application: `f $ x` is the call
/// `f (x)` and lowers to exactly that. The parser therefore builds an ordinary
/// `Call` and records the spelling in `is_dollar`, which is the only thing
/// standing between this printer and the defect that found it — formatting
/// `examples/application.sdx` rewrote every `$` in the file into the
/// parentheses `$` exists to avoid, and a construct the formatter deletes is a
/// construct nobody can use.
///
/// # Both sides print bare, and the caller parenthesises
///
/// Neither side takes a parenthesis of its own:
///
/// ```text
/// twice $ show 7        the argument is a whole application
/// print $ 1 + 2         ...or a whole binary expression
/// a $ b $ c             ...or another `$`, which is right-associativity
/// f a $ b               the *callee* is an application, and keeps no brackets
/// ```
///
/// Each of those re-parses to the node it was printed from, because `$` binds
/// looser than everything but ascription: whatever the parser read to the left
/// of the `$` is what printing the left side at power 0 produces, and the same
/// holds on the right.
///
/// The one side that *does* come out bracketed is a form that would otherwise
/// swallow the `$` — `(\x -> x) $ 3`, where the author's parenthesis is in the
/// tree and `can_drop_paren` declines to drop it because a lambda body extends
/// as far right as it can.
///
/// A `$`-spelled call sitting in a tighter context is parenthesised by that
/// context rather than here, which is the convention the `Binary` arm already
/// follows: `power_of` reports power 0 for this form, so `operand`, `argument`
/// and `receiver` all wrap it.
pub(crate) fn dollar_text(callee: &Expr, arg: &Expr, indent: usize, source: &str) -> String {
  let left = expr_text(callee, 0, indent, source);
  let right = expr_text(arg, 0, indent, source);
  format!("{} $ {}", left, right)
}
