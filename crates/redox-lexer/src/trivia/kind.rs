/// What a piece of [`crate::Trivia`] is.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 3 extends plan 01's two
/// variants with the doc-comment and fence forms of doc 03 §4.1.
///
/// Doc comments are **trivia, not tokens**. They attach to the item they
/// precede exactly as ordinary comments do, so plan 23 can translate them to
/// `///` and `//!` without the grammar knowing they exist. Emerald made the
/// opposite choice — a `doc: Option<String>` field on six AST node kinds — and
/// its formatter can round-trip only those six, losing every other comment.
/// Trivia is the general mechanism; a field on a node is the special case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriviaKind {
  /// `#` — an ordinary comment, on its own line or trailing code. The stored
  /// text excludes the `#` and one optional following space.
  LineComment,
  /// `##` — an outer doc comment, becoming `///`.
  DocComment,
  /// `##!` — an inner doc comment, becoming `//!`.
  InnerDocComment,
  /// `### … ###` — a fenced multi-line comment, becoming `/* … */`. The stored
  /// text is the body between the fences.
  FencedComment,
  /// A run of one or more blank lines. The run's length is not recorded,
  /// because the emitter collapses any run to a single blank line and nothing
  /// in the language depends on the count.
  BlankLines,
}

impl TriviaKind {
  /// Whether this trivium carries documentation, which the emitter renders as
  /// `///` or `//!` rather than `//`.
  pub fn is_doc(&self) -> bool {
    matches!(self, Self::DocComment | Self::InnerDocComment)
  }
}
