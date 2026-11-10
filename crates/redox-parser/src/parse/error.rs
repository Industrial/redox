use redox_diag::{Code, Span};

/// A parse error, carrying enough to report against the `.sdx` source.
///
/// The span is mandatory (doc 06 §6 rule 3), and plan 05 added the `code`:
/// assigned where the error is **constructed**, never matched from message
/// text at render time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
  pub code: Code,
  pub message: String,
  pub span: Span,
  /// A second line, for the reason a rule exists rather than its remedy.
  ///
  /// `None` for almost every parse error, and deliberately so: a grammar
  /// error's remedy is short enough to live in the message, which is why
  /// `RDX0111`'s three readings each end in *"— write `data` …"* rather than
  /// carrying a help. The field exists for the case where the remedy is
  /// obvious and the **reason** is not — `RDX0112`, where moving the `test`
  /// out is clear and *why Redox refuses it at all* is a fact about rustc's
  /// test collector that nothing in the source hints at.
  ///
  /// [`LexError`](redox_lexer::LexError) has carried one since plan 05 and
  /// `diagnose` already passed it through for the lexical half; this is the
  /// syntactic half of the same field.
  pub help: Option<String>,
}

impl ParseError {
  /// The common case: a token appeared where the grammar wanted another.
  pub(crate) fn expected(message: impl Into<String>, span: Span) -> Self {
    Self {
      code: Code::ExpectedToken,
      message: message.into(),
      span,
      help: None,
    }
  }

  /// An error with a code other than [`Code::ExpectedToken`].
  pub(crate) fn with(code: Code, message: impl Into<String>, span: Span) -> Self {
    Self {
      code,
      message: message.into(),
      span,
      help: None,
    }
  }

  /// An error whose **reason** needs saying as well as its remedy.
  pub(crate) fn with_help(
    code: Code,
    message: impl Into<String>,
    help: impl Into<String>,
    span: Span,
  ) -> Self {
    Self {
      code,
      message: message.into(),
      span,
      help: Some(help.into()),
    }
  }
}
