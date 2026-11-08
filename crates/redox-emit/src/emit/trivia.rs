pub(crate) mod line;

use crate::emit::pad::pad;
use crate::emit::sink::Sink;
use crate::emit::trivia::line::line;
use redox_ast::{Trivia, TriviaKind};
use std::fmt::Write as _;

/// Emit leading trivia at `indent`.
///
/// A blank-line run collapses to a single blank line, and never opens a file
/// or a block, which would be noise and would break byte-stability against a
/// blessed golden file.
///
/// The comment forms map onto Rust's, per `docs/03 - Redox Surface Design.md`
/// §11.4. Plan 02 added the doc-comment and fence kinds to the lexer, which
/// made this match non-exhaustive; handling them here rather than adding a
/// `_ => {}` arm is deliberate — a wildcard would silently drop a doc comment,
/// which is the exact defect doc 02 §10.1 exists to prevent.
pub(crate) fn emit_trivia(out: &mut Sink<'_>, trivia: &[Trivia], indent: usize) {
  for t in trivia {
    // A comment the author wrote has a source position of its own, so the
    // generated comment line maps to it. Plan 06's totality test found this:
    // leaving trivia unmarked made every comment line in the output
    // "synthetic", which is false — the author wrote it, and rustc can
    // complain about a doc comment.
    match t.kind {
      TriviaKind::BlankLines => out.unmark(),
      _ => out.mark(t.span),
    }
    match t.kind {
      TriviaKind::LineComment => line(out, indent, "//", &t.text),
      TriviaKind::DocComment => line(out, indent, "///", &t.text),
      TriviaKind::InnerDocComment => line(out, indent, "//!", &t.text),
      TriviaKind::FencedComment => {
        pad(out, indent);
        let _ = writeln!(out, "/* {} */", t.text);
      }
      TriviaKind::BlankLines => {
        if !out.is_empty() && !out.ends_with("{\n") && !out.ends_with("\n\n") {
          out.push('\n');
        }
      }
    }
  }
}
