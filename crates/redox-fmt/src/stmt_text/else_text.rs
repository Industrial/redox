use crate::stmt_text::block_text::block_text;

use crate::stmt_text::code_text::code_text;

use crate::stmt_text::indent_text::indent_text;

use redox_ast::{Block, Expr};

/// An `if` statement's `else` branch, or nothing.
///
/// # `else if` is a chain, not a nested block
///
/// The AST has no `else if`: the parser builds `otherwise: Some(Block)` whose
/// one statement is another `If`. Printed literally that is
///
/// ```text
/// if a
///   ...
/// else
///   if b
///     ...
/// ```
///
/// which is correct Redox and is not what anyone writes.
/// `examples/control.sdx` writes `else if n < 0`, and doc 18 describes the
/// form as chaining *"as Rust's does — an `else` whose branch is another
/// `if`"*. So the chain is reconstructed here, and the condition for doing it
/// is exact: the block holds **one** statement, that statement is an `If`, and
/// it carries **no trivia**.
///
/// The trivia test is the part worth keeping. A comment above the inner `if`
/// belongs on its own line, and `else if -- why` is not a line Redox can
/// write — collapsing the chain would have to move the comment or drop it, and
/// dropping a comment is doc 02 §3.2's defect. So a commented inner `if` stays
/// a nested block, which is uglier and right.
pub(crate) fn else_text(r#otherwise: &Option<Block>, indent: usize, source: &str) -> String {
  // A `guard` where the Rust wrote `let … else`, which is the same early
  // return: doc 30 §3.1's pattern test with the scrutinee first.
  let Some(block) = r#otherwise else {
    return String::new();
  };
  let pad = indent_text(indent);
  // A four-term let-chain: a slice pattern, then three predicates. The
  // `matches!` in the middle is a nested `case`, which plan 64 §6 explains —
  // its second argument is a pattern, so no macro call can carry it.
  let sole_if = match block.stmts.as_slice() {
    [only] => match only.expr {
      Expr::If { .. } => only.leading.is_empty() && only.trailing.is_none(),
      _ => false,
    },
    _ => false,
  };
  if sole_if {
    let only = &block.stmts[0];
    return format!("\n{}else {}", pad, code_text(&only.expr, indent, source));
  }
  format!("\n{}else\n{}", pad, block_text(block, indent + 1, source))
}
