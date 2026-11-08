use crate::lexer::Lexer;
use crate::{LexError, Token, TokenKind};

/// Lex an expression fragment, offsetting every span by `base`.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 4. This is what makes
/// interpolation recursive: an interpolation body goes through the same
/// scanners as top-level source, so `"#{f(x, "y")}"` works and nothing needs a
/// second tokenizer.
///
/// Structural tokens are dropped — a fragment has no statements to separate
/// and no blocks to open — and spans are shifted so they index the *original*
/// file rather than the fragment, which is what lets a diagnostic inside an
/// interpolation point at the right column.
pub(crate) fn lex_fragment(src: &str, base: usize) -> Result<Vec<Token>, LexError> {
  let mut toks = Lexer::new(src).run()?;
  toks.retain(|t| !t.kind.is_structural() && t.kind != TokenKind::Eof);
  for t in &mut toks {
    t.span = (t.span.0 + base, t.span.1 + base);
    for trivium in &mut t.leading {
      trivium.span = (trivium.span.0 + base, trivium.span.1 + base);
    }
  }
  Ok(toks)
}
