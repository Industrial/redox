/// The token set for plan 01's slice.
///
/// Deliberately minimal. Continuation suppression inside delimiters, the `#`
/// longest-match table of doc 03 §4.1, interpolation and `"""` strings are
/// plan 02.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
  Ident(String),
  /// A string literal, stored **raw, including its delimiters**. Escape
  /// decoding and interpolation are plan 23; storing the source text verbatim
  /// lets the emitter reproduce it byte-for-byte and keeps this slice free of
  /// an escape table it would only have to redo.
  StrLit(String),
  Eq,
  Arrow,
  /// A logical line break terminating a statement.
  Newline,
  Indent,
  Dedent,
  Eof,
}

impl TokenKind {
  /// Whether this token is structural — emitted at a position the parser does
  /// not read trivia from.
  ///
  /// Structural tokens never claim trivia. Letting a `Dedent` absorb a pending
  /// comment loses it silently, which is exactly how a comment between two
  /// top-level items, or at end of file, used to disappear. `Eof` counts as
  /// non-structural so a trailing comment has somewhere to land.
  pub(crate) fn is_structural(&self) -> bool {
    matches!(self, Self::Indent | Self::Dedent | Self::Newline)
  }
}
