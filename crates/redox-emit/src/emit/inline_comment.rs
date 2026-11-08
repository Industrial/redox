use crate::emit::sink::Sink;
use redox_ast::{Trivia, TriviaKind};
use std::fmt::Write as _;

/// Append a trailing `// …` comment to the current line, if there is one.
pub(crate) fn emit_inline_comment(out: &mut Sink<'_>, trivia: Option<&Trivia>) {
  if let Some(t) = trivia
    && t.kind == TriviaKind::LineComment
  {
    if t.text.is_empty() {
      out.push_str(" //");
    } else {
      let _ = write!(out, " // {}", t.text);
    }
  }
}
