pub(crate) mod kind;

use crate::{Span, TokenKind, Trivia};

/// A token, its span, the trivia preceding it, and whether whitespace
/// immediately preceded it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
  pub kind: TokenKind,
  pub span: Span,
  /// Whether whitespace immediately preceded this token.
  ///
  /// Unused by plan 01's slice and free to collect here. Doc 03 §5.1's call
  /// grammar needs it to tell `f(x)` from `f (x)`, and §5.4's
  /// `(-1)`-versus-`(- 1)` rule needs it to tell a negative literal from a
  /// subtraction section.
  pub ws_before: bool,
  pub leading: Vec<Trivia>,
  /// Trivia on the same line, after this token. Only ever set on the
  /// [`TokenKind::Newline`] that closes the line.
  pub trailing: Option<Trivia>,
}
