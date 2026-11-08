use crate::lexer::Lexer;
use crate::{LexError, Token};

pub(crate) mod error;

/// Tokenize `src`.
///
/// Returns every token in order, ending with [`crate::TokenKind::Eof`].
/// Trivia is attached to the token that follows it; trivia at the end of the
/// file is attached to `Eof`.
pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
  Lexer::new(src).run()
}
