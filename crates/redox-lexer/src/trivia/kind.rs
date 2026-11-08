/// What a piece of [`crate::Trivia`] is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriviaKind {
  /// A `#` comment, either on its own line or trailing a line of code. The
  /// stored text excludes the `#` and one optional following space.
  LineComment,
  /// A run of one or more blank lines. The run's length is not recorded,
  /// because the emitter collapses any run to a single blank line and nothing
  /// in the language depends on the count.
  BlankLines,
}
