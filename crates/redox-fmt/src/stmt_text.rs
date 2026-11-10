/// A binding — `n = 0`, `mut n = 0`, `n : u64 = 0`.
pub(crate) mod binding_text;

/// A block's statements, one per line.
pub(crate) mod block_text;

/// One `case` arm — `pattern -> body`, with an optional guard.
pub(crate) mod case_arm_text;

/// A block element's code, with its first line unindented.
pub(crate) mod code_text;

/// A condition — operands joined by `&&`, each an expression or a pattern test.
pub(crate) mod condition_text;

/// An `if` statement's `else` branch, or nothing.
pub(crate) mod else_text;

/// The leading whitespace for a given depth.
pub(crate) mod indent_text;

/// A loop label, with its trailing space — or nothing.
pub(crate) mod label_text;

/// One trivium as Redox source: a comment, an attribute, or a blank line.
pub(crate) mod trivia_text;

use crate::item_text::item_text;

use crate::stmt_text::code_text::code_text;

use crate::stmt_text::indent_text::indent_text;

use crate::stmt_text::trivia_text::trivia_text;

use redox_ast::Expr;

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
/// `code_text` deliberately does not: that function returns its first line
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
  // Plan 55 leaf 5. A declaration writes its own padding and its own leading
  // trivia — `item_text` does both, because a module-scope declaration has no
  // caller to do it — so it is written here rather than through `code_text`,
  // which runs *after* the pad below. `emit_stmt` routes an item before its
  // own `pad` call for the identical reason.
  //
  // Going through `code_text` and stripping the pad was the first attempt. It
  // works only for an item with no leading trivia: with any, the output starts
  // with the trivia's newline, the strip misses, and a blank line between two
  // nested declarations comes out as the pad alone.
  let Expr::Item { item, .. } = &stmt.expr else {
    let mut lines: Vec<String> = Vec::new();
    for trivium in &stmt.leading {
      lines.push(trivia_text(trivium, indent));
    }
    let mut code = format!(
      "{}{}",
      indent_text(indent),
      code_text(&stmt.expr, indent, source)
    );
    // **Two** spaces before a trailing comment. The corpus writes two in four
    // places and one in a fifth, and nothing distinguishes the fifth — so the
    // majority decides and `examples/comments.sdx` gains a space. Picking one
    // the other way was the first attempt, on the reasoning that the file
    // about comments should be the authority on comments; four files against
    // one is the better argument.
    if let Some(trailing) = &stmt.trailing {
      code.push_str("  ");
      code.push_str(trivia_text(trailing, 0).trim_start());
    }
    lines.push(code);
    return lines.join("\n");
  };
  item_text(item, indent, source)
}
