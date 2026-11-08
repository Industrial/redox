use redox_lexer::Span;

/// An identifier and where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
  pub text: String,
  pub span: Span,
}
