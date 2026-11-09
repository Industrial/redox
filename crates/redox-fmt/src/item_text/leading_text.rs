use crate::stmt_text::trivia_text::trivia_text;
use redox_ast::Item;

/// An item's leading trivia, as complete lines each ending in a newline — or
/// nothing.
///
/// Lines rather than a joined block, so the caller can push the item's own
/// indentation straight after it. That is what makes an item inside a `module`
/// line up with its comment.
///
/// `redox_ast::leading` is the accessor leaf 1 added for exactly this, so the
/// eleven-arm match over item kinds lives there and not once per caller.
pub(crate) fn leading_text(item: &Item, indent: usize) -> String {
  let mut out = String::new();
  for trivium in redox_ast::leading(item) {
    out.push_str(&trivia_text(trivium, indent));
    out.push('\n');
  }
  out
}
