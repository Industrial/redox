use crate::lexer::cursor::Cursor;
use crate::lexer::dedent_block::dedent_block;

/// The layout-delimited body of a `rust` block, taken verbatim — plan 64
/// leaf 4.
///
/// ```text
/// markup = rust
///   html! { <div class="x">{ name }</div> }
/// ```
///
/// Returns `None` when the `rust` keyword is followed by anything on its own
/// line, which is the `"""` string form and stays [`crate::verbatim_triple`]'s
/// — so this is additive and plan 45 leaf 1's spelling keeps working
/// unchanged.
///
/// # Why this is the lexer's job and not the parser's
///
/// A token-stream DSL's input is **not Redox**, and the parser never gets to
/// say so: by the time it runs, the lexer has already tried to tokenise
/// `<div class="x">` and may have failed outright. Slicing the region by span
/// after the fact cannot undo a lexical error that was reported before the
/// parser existed. So the region has to be skipped where tokens are made.
///
/// That is also why plan 64 §1.4's *"the mechanism exists and is tested"* was
/// overstated and §11.5 corrected it: what existed was the `"""` **string**
/// form, where the lexer's own string scanner does the skipping.
///
/// # Where the cursor is left, and why it matters
///
/// On the **newline that ends the last body line**, not past it. The lexer's
/// `step` sees that newline on its next turn and runs `end_of_line` and
/// `start_of_line` in the ordinary way, which is what syncs the dedent and
/// closes the enclosing blocks. Consuming it here would leave the next line's
/// indentation unread and the off-side rule one level out.
///
/// `base` is the indentation the enclosing block is at — `*indents.last()` —
/// and the body must be deeper than it. A body that is not is not a block at
/// all, so this returns `None` and the caller reports `rust` as an ordinary
/// keyword with nothing after it.
pub(crate) fn scan_rust_block(cur: &mut Cursor<'_>, base: usize) -> Option<String> {
  let mark = cur.mark();
  cur.eat_spaces();
  if cur.peek() != Some('\n') {
    cur.reset_to(mark);
    return None;
  }

  // Every line of the body, including the blank ones between its statements.
  // `end` tracks the last **non-blank** line's newline, so a trailing blank
  // line is left for the off-side rule rather than swallowed into the block.
  let body_start = cur.pos() + 1;
  let mut end = None;
  loop {
    let line_mark = cur.mark();
    if cur.peek() != Some('\n') {
      break;
    }
    cur.bump();
    let width = cur.eat_spaces();
    if cur.peek() == Some('\n') {
      // A blank line belongs to the block only if something deeper follows,
      // which the next turn of this loop decides.
      continue;
    }
    if cur.is_eof() || width <= base {
      cur.reset_to(line_mark);
      break;
    }
    cur.eat_while(|c| c != '\n');
    end = Some(cur.pos());
  }

  let end = end?;
  cur.reset_to(end);
  Some(dedent_block(cur.slice((body_start, end))))
}
