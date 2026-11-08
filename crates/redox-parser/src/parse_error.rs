use redox_lexer::Span;

/// A parse error, carrying enough to report against the `.sdx` source.
///
/// `miette` rendering is plan 05. The span is mandatory regardless — doc 06 §5
/// rule 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
  pub message: String,
  pub span: Span,
}
