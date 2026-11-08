pub(crate) mod ident;

use crate::Ident;
use redox_lexer::Span;

/// An expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
  /// A call. Plan 01's slice produces only the parenless, single-argument,
  /// statement-initial form; plan 19 generalises it to doc 03 §5.1's full call
  /// grammar.
  Call {
    callee: Box<Expr>,
    args: Vec<Expr>,
    span: Span,
  },
  /// A string literal, stored **raw, including its delimiters** — see
  /// `redox_lexer::TokenKind::StrLit`. Escape decoding is plan 23.
  StrLit {
    value: String,
    span: Span,
  },
  Ident(Ident),
}

impl Expr {
  pub fn span(&self) -> Span {
    match self {
      Expr::Call { span, .. } | Expr::StrLit { span, .. } => *span,
      Expr::Ident(i) => i.span,
    }
  }
}
