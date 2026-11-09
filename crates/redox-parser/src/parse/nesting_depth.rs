use redox_lexer::{Token, TokenKind};

/// The deepest bracket nesting in a token stream.
///
/// Counts `(`, `[` and `{` together, because what the parser's stack cares
/// about is how many frames deep a nested expression goes and every bracketing
/// form costs one.
///
/// A linear scan rather than a counter threaded through the recursive descent:
/// the question is about the **input**, the answer is wanted before parsing
/// starts, and a pre-pass cannot be defeated by an error-recovery path that
/// forgets to decrement.
pub(crate) fn nesting_depth(toks: &[Token]) -> usize {
  let mut depth = 0usize;
  let mut deepest = 0usize;
  for token in toks {
    match token.kind {
      TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
        depth += 1;
        deepest = deepest.max(depth);
      }
      TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
        depth = depth.saturating_sub(1);
      }
      _ => {}
    }
  }
  deepest
}
