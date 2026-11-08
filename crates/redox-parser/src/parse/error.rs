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
}

impl ParseError {
  /// The common case: a token appeared where the grammar wanted another.
  pub(crate) fn expected(message: impl Into<String>, span: Span) -> Self {
    Self {
      code: Code::ExpectedToken,
      message: message.into(),
      span,
    }
  }

  /// An error with a code other than [`Code::ExpectedToken`].
  pub(crate) fn with(code: Code, message: impl Into<String>, span: Span) -> Self {
    Self {
      code,
      message: message.into(),
      span,
    }
  }
}
