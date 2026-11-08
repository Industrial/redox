use crate::Span;

/// A lexical error, carrying enough to report against the `.sdx` source.
///
/// `miette` rendering is plan 05; a plain message with a position is what plan
/// 01 needs. The span is mandatory regardless — see
/// `docs/06 - Redox Code Architecture.md` §5 rule 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
  pub message: String,
  pub span: Span,
  pub line: usize,
  pub col: usize,
}
