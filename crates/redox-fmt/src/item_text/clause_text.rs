pub(crate) mod inline_body;

use crate::expr_text::expr_text;
use crate::item_text::clause_text::inline_body::inline_body;
use crate::item_text::same_line::same_line;
use crate::pattern_text::pattern_argument::pattern_argument;
use crate::stmt_text::block_text::block_text;
use crate::stmt_text::indent_text::indent_text;
use crate::stmt_text::stmt_text;
use redox_ast::{Clause, ClauseBody};

/// One equation — `add a b = a + b`, or a guard ladder, with its `where`.
///
/// `name` is passed in because the clause does not hold it: the parser's
/// clause span starts *after* the function's name, which belongs to the
/// declaration.
///
/// The patterns print as **arguments**, for the same reason a call's do —
/// juxtaposition is greedy, so `fib (n - 1)` keeps its parentheses.
///
/// Whether a body is inline or indented is the author's and is recovered from
/// the block's span; `same_line` records why the tree cannot answer it.
pub(crate) fn clause_text(clause: &Clause, name: &str, indent: usize, source: &str) -> String {
  let mut head = name.to_string();
  for pattern in &clause.patterns {
    head.push(' ');
    head.push_str(&pattern_argument(pattern));
  }

  let body = match &clause.body {
    // §3 row 10: one `|` per line, which is doc 18 §2.3's own example. A
    // ladder is always indented, so there is no inline shape to preserve.
    ClauseBody::Guarded(guards) => {
      let pad = indent_text(indent + 1);
      let printed: Vec<String> = guards
        .iter()
        .map(|guard| {
          let test = match &guard.condition {
            Some(condition) => expr_text(condition, 0, indent + 1, source),
            // The catch-all. In Haskell `otherwise` is an ordinary binding;
            // here it is the spelling the parser reads, and both
            // `examples/guards.sdx` and `examples/testing.sdx` end their
            // ladders with it.
            None => "otherwise".to_string(),
          };
          let arm = if same_line(source, guard.span.0, guard.body.span.0) {
            format!(" = {}", inline_body(&guard.body, indent + 1, source))
          } else {
            format!(" =\n{}", block_text(&guard.body, indent + 2, source))
          };
          format!("{pad}| {test}{arm}")
        })
        .collect();
      format!("\n{}", printed.join("\n"))
    }
    ClauseBody::Plain(block) => {
      if same_line(source, clause.span.0, block.span.0) {
        format!(" = {}", inline_body(block, indent, source))
      } else {
        format!(" =\n{}", block_text(block, indent + 1, source))
      }
    }
  };

  let wheres = if clause.wheres.is_empty() {
    String::new()
  } else {
    let printed: Vec<String> = clause
      .wheres
      .iter()
      .map(|stmt| stmt_text(stmt, indent + 2, source))
      .collect();
    format!("\n{}where\n{}", indent_text(indent + 1), printed.join("\n"))
  };

  format!("{head}{body}{wheres}")
}
