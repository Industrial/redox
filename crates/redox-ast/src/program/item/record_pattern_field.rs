use crate::{Ident, Pattern};

/// One field of a record pattern — `x` or `x = 0`. Plan 33.
///
/// Doc 03 §6.3's *"field pun"*: `Point { x, y }` binds both, which is Rust's
/// own shorthand and needs no spelling of its own. A named field carries a
/// pattern, so `Point { x = Some v }` nests like anything else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordPatternField {
  pub name: Ident,
  /// `None` is a pun — the field binds under its own name.
  pub pattern: Option<Pattern>,
}

impl RecordPatternField {
  /// The text to write in a Rust pattern.
  ///
  /// `=` becomes `:`, which is the one character doc 18 §2.4's spelling
  /// changes — plan 33 §1.2.
  pub fn rust_text(&self) -> String {
    match &self.pattern {
      Some(pattern) => format!("{}: {}", self.name.text, pattern.rust_text()),
      None => self.name.text.clone(),
    }
  }
}
