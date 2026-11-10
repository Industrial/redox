use crate::lint::comment_pragma::comment_pragma;
use redox_diag::{Lint, Span};

/// Every comment-form suppression pragma in a source file, with the lint it
/// names, for `RDX1002` — plan 53 leaf 5, parametrised by plan 54 leaf 3.
///
/// The lint travels with the span because the deprecation message quotes both
/// spellings, so a pragma for `shadow` must not be told to write the one for
/// `unwrap`.
///
/// **Lexical, and completely so.** Unlike the suppression *scope*, this needs
/// no tree: every trivium reaches a token's `leading` or `trailing`, so a
/// token scan sees all of them, including the ones on a construct this
/// release's suppression walk does not reach. A deprecation that missed a
/// site would leave an author migrating twice.
///
/// It runs on a file that does not parse for the same reason, which is what
/// the language server wants: the lint is still an answer about the text.
pub(crate) fn comment_pragma_sites(source: &str) -> Vec<(Span, Lint)> {
  let Ok(tokens) = redox_lexer::lex(source) else {
    // Nothing to say about a file whose own lexical error is the diagnostic.
    return Vec::new();
  };

  tokens
    .iter()
    .flat_map(|t| t.leading.iter().chain(t.trailing.as_ref()))
    .filter_map(|trivium| comment_pragma(trivium).map(|lint| (trivium.span, lint)))
    .collect()
}
