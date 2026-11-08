use crate::Item;
use redox_lexer::{Span, Trivia};

/// A whole source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
  pub items: Vec<Item>,
  pub span: Span,
  /// Trivia after the last item — a trailing comment at end of file.
  pub trailing: Vec<Trivia>,
}
