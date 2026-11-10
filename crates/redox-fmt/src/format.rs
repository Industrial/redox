pub(crate) mod trimmed_tail;

use crate::format::trimmed_tail::trimmed_tail;

use crate::item_text::item_text;

use crate::stmt_text::trivia_text::trivia_text;

use redox_ast::Program;

/// Format a parsed program back into canonical Redox.
///
/// # Leaf 2 replaced the span slicer
///
/// Leaf 1 cut each item out of the source between item boundaries and
/// canonicalised only the gaps between them. Every printer since has replaced
/// one slice with a real printer, and leaf 2 was the last: an item is now
/// printed from the tree, so this function is a join.
///
/// The three models leaf 1 needed — item boundaries rather than spans
/// (§10's note on a comment emitted twice), `trimmed_range`, and
/// `blank_line_in` — are all gone with it. The reason they are not merely
/// refactored away is worth recording: **a blank line is a trivium**, so the
/// gaps leaf 1 measured out of the source were already in the tree. Leaf 1
/// measured them because it was not printing trivia yet, and leaf 4 made that
/// unnecessary rather than easier.
///
/// # What survives from leaf 1
///
/// Its three file-level rules, now as consequences rather than code. A run of
/// blank lines collapses to one because `TriviaKind::BlankLines` does not
/// record a run's length. There is exactly one trailing newline because each
/// item is followed by one. No line has trailing whitespace because every
/// printer writes its own indentation and a blank line is written empty.
///
/// Leading blank lines are the one rule that still needs saying, and it is
/// said here: a `BlankLines` trivium on the first item would otherwise open
/// the file with an empty line.
///
/// # One construct is still sliced
///
/// A function's **signature line**, for the reason `signature_line` records at
/// length: `parser::generalise` rewrites a signature's type variables into
/// Rust's uppercase convention in place, so the tree cannot say what the
/// author wrote. That is criterion 7 doing its job, and it is the one thing
/// standing between this formatter and reading no source at all.
pub fn format(program: &Program, source: &str) -> String {
  let mut out = String::new();
  let mut first = true;
  for item in &program.items {
    let mut text = item_text(item, 0, source);
    // A file does not open with a blank line, whatever the author wrote.
    if first {
      first = false;
      text = String::from(text.trim_start_matches('\n'));
    }
    // `mut text` and `:=` where the first port wrote `text = if first …`, and
    // the pipelines are the reason for both. Plan 69's second sighting: a
    // binding whose **value** is a block — an `if`, a `case` — swallows a
    // following statement that opens with `.name`. The value goes through the
    // Pratt loop, a block ends on a `Dedent` that does not stop it, and so
    // `postfix` reads the `.` as field access on the block; `RDX0115` then
    // refuses the method call. An `if` in *statement* position does not do
    // this, which is what `expr_text/macro_call_text.sdx` relies on, and the
    // fixture is `tests/ui/block_valued_binding_eats_the_next_statement`.
    //
    // Reassigning removes the block value; `recv |> .name args` — the
    // spelling `RDX0115`'s own help text offers — then reads better than two
    // leading dots for two lines that are both about `out`.
    out.push_str(&text);
    out.push('\n');
  }
  for trivium in &program.trailing {
    out.push_str(&trivia_text(trivium, 0));
    out.push('\n');
  }
  // A blank line **before** an end-of-file comment is content and stays;
  // blank lines at the very end are not. `trimmed_tail` records why that
  // decision is made on the finished text rather than on the trivium.
  trimmed_tail(out)
}
