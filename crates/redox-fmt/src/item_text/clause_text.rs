/// A body that sits on the equation line, with no indentation of its own.
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
    ClauseBody::Guarded(guards) => {
      let pad = indent_text(indent + 1);
      // `r#guard` because `guard` is plan 59's keyword and this is a lambda
      // *parameter* — the half of the keyword surface plan 68's survey missed,
      // which covered fields and modules only.
      let printed: Vec<String> = guards
        .iter()
        .map(|r#guard| {
          // bare `test = …` earns `RDX0112` for being a nested one. Third
          // site of plan 69's keyword finding, and the first on a plain
          // local rather than a field or a parameter.
          let r#test = match &r#guard.condition {
            Some(condition) => expr_text(condition, 0, indent + 1, source),
            None => String::from("otherwise"),
          };
          let arm = if same_line(source, r#guard.span.0, r#guard.body.span.0) {
            format!(" = {}", inline_body(&r#guard.body, indent + 1, source))
          } else {
            format!(" =\n{}", block_text(&r#guard.body, indent + 2, source))
          };
          format!("{}| {}{}", pad, r#test, arm)
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
  format!("{}{}{}", head, body, wheres)
}
