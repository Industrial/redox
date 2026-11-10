use crate::expr_text::expr_text;
use crate::pattern_text::pattern_text;
use crate::stmt_text::block_text::block_text;
use crate::stmt_text::condition_text::condition_text;
use crate::stmt_text::indent_text::indent_text;
use redox_ast::{CaseArm, Expr};

/// One `case` arm — `pattern -> body`, with an optional guard.
///
/// §3 row 7: **one space after the `->`**, never padding to align several
/// arms' bodies. The plan's reason is that alignment breaks on the next edit;
/// the stronger one is that it makes one arm's change show up as a diff in
/// every other arm.
///
/// The guard is a full condition, so `n | n > 65 -> "senior"` and
/// `y | Some (Some v) = y -> v` are one form with different operands.
///
/// # A block body prints with no marker, which migrates `-> do`
///
/// Plan 55 leaf 3 made an indented block an arm body, and it is the same
/// node as `-> do` — `Expr::Do` with no label — because the two spellings
/// mean the same thing and emit the same Rust. The parser therefore cannot
/// tell them apart, so the printer has to choose one, and it writes the
/// marker-free form: the deprecated spelling is what leaf 7 removes, and a
/// formatter that printed it would be writing output its own next row
/// cannot parse. Doc 02 §4.5's migration tool, doing the migration.
///
/// A **labelled** block keeps `do 'name`, which is the one thing the marker
/// still carries (§1.7).
pub(crate) fn case_arm_text(arm: &CaseArm, indent: usize, source: &str) -> String {
  let guard = match &arm.guard {
    Some(condition) => format!(" | {}", condition_text(condition, indent, source)),
    None => String::new(),
  };
  let head = format!(
    "{}{}{} ->",
    indent_text(indent),
    pattern_text(&arm.pattern),
    guard
  );
  match &arm.body {
    Expr::Do {
      label: None, body, ..
    } => {
      format!("{head}\n{}", block_text(body, indent + 1, source))
    }
    body => format!("{head} {}", expr_text(body, 0, indent, source)),
  }
}
