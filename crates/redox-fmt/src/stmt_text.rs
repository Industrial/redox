pub(crate) mod binding_text;
pub(crate) mod block_text;
pub(crate) mod case_arm_text;
pub(crate) mod condition_text;
pub(crate) mod else_text;
pub(crate) mod indent_text;
pub(crate) mod kind_text;
pub(crate) mod label_text;
pub(crate) mod trivia_text;

use crate::stmt_text::indent_text::indent_text;
use crate::stmt_text::kind_text::kind_text;
use crate::stmt_text::trivia_text::trivia_text;
use redox_ast::Stmt;

/// One statement in canonical Redox, with its trivia — plan 34, leaf 4,
/// second half.
///
/// `indent` is the statement's depth in levels, not spaces. §3 row 11 fixes a
/// level at **two spaces**, and `indent_text` is the only place that conversion
/// happens.
///
/// # What this returns, exactly
///
/// Complete lines, each at its own depth, with **no trailing newline**. The
/// first line carries this statement's indentation like every other, which
/// `kind_text` deliberately does not: that function returns its first line
/// unindented so a caller can put it after `else ` or `= `, and this function
/// is where a statement standing on its own line gets its prefix.
///
/// # Trivia is printed, not sliced
///
/// Leading trivia becomes its own lines above the statement; a trailing
/// comment is appended to the last line. That makes this the first printer in
/// the plan that can lose a comment, and criterion 4 is the assertion that it
/// does not — counted rather than inspected, because a comment that survives
/// into the wrong place is still a comment the count finds.
pub fn stmt_text(stmt: &Stmt, indent: usize, source: &str) -> String {
  let mut lines: Vec<String> = Vec::new();
  for trivium in &stmt.leading {
    lines.push(trivia_text(trivium, indent));
  }
  let mut code = format!(
    "{}{}",
    indent_text(indent),
    kind_text(&stmt.kind, indent, source)
  );
  if let Some(trailing) = &stmt.trailing {
    // One space before a trailing comment. The corpus writes two in four
    // places and one in a fifth, and the fifth is `comments.sdx` — the file
    // whose whole subject is comments — so that is the tiebreak §1.5 gives.
    code.push(' ');
    code.push_str(trivia_text(trailing, 0).trim_start());
  }
  lines.push(code);
  lines.join("\n")
}
