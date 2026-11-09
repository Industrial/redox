pub(crate) mod with_marker;

use crate::stmt_text::indent_text::indent_text;
use crate::stmt_text::trivia_text::with_marker::with_marker;
use redox_lexer::{Trivia, TriviaKind};

/// One trivium as Redox source: a comment, an attribute, or a blank line.
///
/// The inverse of what the emitter does, and the markers are doc 18 §5.1's:
/// `--` for a comment, `-- |` and `-- !` for the two doc forms, `{- … -}` for
/// a fence. The stored text excludes its marker — and, for a line comment, one
/// optional following space — so this writes the marker and one space back.
///
/// A blank line is **empty**, not indented. An indented blank line is trailing
/// whitespace, which leaf 1 already made a thing this formatter removes.
pub(crate) fn trivia_text(trivium: &Trivia, indent: usize) -> String {
  let pad = indent_text(indent);
  match trivium.kind {
    TriviaKind::BlankLines => String::new(),
    TriviaKind::LineComment => with_marker(&pad, "--", &trivium.text),
    TriviaKind::DocComment => with_marker(&pad, "-- |", &trivium.text),
    TriviaKind::InnerDocComment => with_marker(&pad, "-- !", &trivium.text),
    // The stored text is the body with its surrounding space already taken
    // off, so the fence puts one back on each side — `{-a comment-}` is what
    // came out before, and it is not what anyone writes.
    TriviaKind::FencedComment => format!("{pad}{{- {} -}}", trivium.text),
    TriviaKind::Attribute => format!("{pad}#[{}]", trivium.text),
    TriviaKind::InnerAttribute => format!("{pad}#![{}]", trivium.text),
  }
}
