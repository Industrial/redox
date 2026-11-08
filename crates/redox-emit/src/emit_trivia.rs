use crate::pad::pad;
use redox_ast::{Trivia, TriviaKind};
use std::fmt::Write as _;

/// Emit leading trivia at `indent`.
///
/// A comment becomes `//`; a blank-line run collapses to a single blank line,
/// and never opens a file or a block, which would be noise and would break
/// byte-stability against a blessed golden file.
pub(crate) fn emit_trivia(out: &mut String, trivia: &[Trivia], indent: usize) {
  for t in trivia {
    match t.kind {
      TriviaKind::LineComment => {
        pad(out, indent);
        if t.text.is_empty() {
          out.push_str("//\n");
        } else {
          let _ = writeln!(out, "// {}", t.text);
        }
      }
      TriviaKind::BlankLines => {
        if !out.is_empty() && !out.ends_with("{\n") && !out.ends_with("\n\n") {
          out.push('\n');
        }
      }
    }
  }
}
