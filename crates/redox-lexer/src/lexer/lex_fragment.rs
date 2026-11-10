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
///
/// # The error is rebased too, and was not until plan 64
///
/// Every token's span was shifted and the **failure's** was not, so a lexical
/// error inside an interpolation reported an offset into the fragment as
/// though it were an offset into the file: `println! \-> "#{'ab'}"` on line 3
/// blessed to *line 1, column 4*.
///
/// Older than the row that found it — `print "hi #{0x}"` would always have
/// done this — and unwitnessed until plan 64 leaf 6's migration moved two
/// `tests/ui/` fixtures' bad tokens inside an interpolation. [`LexError`]'s
/// own doc comment describes the identical failure one path over, and
/// `split_parts` applies `offset_by` to `scan_escape` and
/// `skip_interpolation`; this was the one call that did not.
pub(crate) fn lex_fragment(src: &str, base: usize) -> Result<Vec<Token>, LexError> {
  let mut toks = Lexer::new(src).run().map_err(|e| e.offset_by(base))?;
  toks.retain(|t| !t.kind.is_structural() && t.kind != TokenKind::Eof);
  for t in &mut toks {
    t.span = (t.span.0 + base, t.span.1 + base);
    for trivium in &mut t.leading {
      trivium.span = (trivium.span.0 + base, trivium.span.1 + base);
    }
  }
  Ok(toks)
}
