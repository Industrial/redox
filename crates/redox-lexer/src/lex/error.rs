use redox_diag::{Code, Span};

/// A lexical error, carrying enough to report against the `.sdx` source.
///
/// The span is mandatory (doc 06 §6 rule 3), and plan 05 added the `code`:
/// assigned where the error is **constructed**, never matched from message
/// text at render time, so improving a message cannot silently change a code.
///
/// There is deliberately no `line`/`col` pair. One was carried here until
/// plan 05's UI corpus caught it lying: an escape error inside a string is
/// raised by a cursor over the string's *body*, so the position it recorded
/// was relative to the body and named the wrong line, while the span — which
/// `split_parts` rebases onto the file — was right all along. A span is the
/// single source of truth; `redox-diag` derives line and column from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
  pub code: Code,
  pub message: String,
  pub span: Span,
  /// A one-line suggestion, shown after the snippet.
  pub help: Option<String>,
}

impl LexError {
  /// Shift this error's span onto the original file.
  ///
  /// A scanner run over a sub-slice — a string's body, an interpolation —
  /// reports offsets into that slice. The caller knows where the slice began
  /// and is the only one who can correct them, which is the same rebasing
  /// [`crate::lexer::lex_fragment::lex_fragment`] does for tokens.
  pub(crate) fn offset_by(mut self, base: usize) -> Self {
    self.span = (self.span.0 + base, self.span.1 + base);
    self
  }
}
