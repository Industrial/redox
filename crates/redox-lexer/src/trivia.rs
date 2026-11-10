/// What a piece of [`crate::Trivia`] is.
pub(crate) mod kind;

use crate::{Span, TriviaKind};

/// Non-semantic source text that must survive to the emitter.
///
/// Comments are trivia, **never discarded**. The generated `.xrs` is a
/// human-facing artifact, so comments have to reach it — doc 02 §10.1 corrects
/// doc 01 §4.1 on this point. Emerald's formatter loses every non-doc comment
/// because its lexer treats them as whitespace; that is the defect this type
/// exists to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trivia {
  pub kind: TriviaKind,
  pub text: String,
  pub span: Span,
}

impl Trivia {
  /// A comment of the given form, with its text and span.
  pub(crate) fn comment(kind: TriviaKind, text: String, span: Span) -> Self {
    Self { kind, text, span }
  }

  /// A run of blank lines, which carries no text of its own.
  pub(crate) fn blank(span: Span) -> Self {
    Self {
      kind: TriviaKind::BlankLines,
      text: String::new(),
      span,
    }
  }
}
