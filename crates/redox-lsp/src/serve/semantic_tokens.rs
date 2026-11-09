pub(crate) mod token_index;

use crate::serve::positions::position_of::position_of as to_position;
use crate::serve::semantic_tokens::token_index::token_index;
use lsp_types::{SemanticToken, SemanticTokens, SemanticTokensResult};

/// The document's tokens, delta-encoded as the protocol wants them.
///
/// Plan 40 §1.4: this is the lexer's output renamed. Doc 02 §5.5 lists
/// semantic tokens because a tree-sitter grammar — row 41 — is a separate and
/// larger job, and the two are independent answers to the same question.
///
/// # The encoding
///
/// Each token is five integers: the line **delta** from the previous token,
/// the character delta (from the previous token's start, or from the line
/// start when the line changed), the length, the type index, and the
/// modifiers. All three of the deltas are relative, which is why a single
/// wrong length silently shifts every later token on the line — and why
/// criterion 6 asserts the encoding by **decoding it back** rather than by
/// eyeballing a number.
///
/// # The tokens that produce nothing
///
/// `Indent`, `Dedent`, `Newline` and `Eof` carry spans and are not text. A
/// loop that trusted the span would emit zero-width tokens, which an editor
/// renders as nothing or as a glitch depending on the editor — plan 40 §1.4
/// names this because it is the one way a mapping table can be wrong without
/// looking wrong.
///
/// A file that does not lex gets no tokens, which is criterion 7: the lexer
/// is the first thing to fail on a half-typed line, and losing all colour for
/// a keystroke is better than a server that stops answering.
pub(crate) fn semantic_tokens(source: &str) -> SemanticTokensResult {
  let Ok(tokens) = redox_lexer::lex(source) else {
    return SemanticTokensResult::Tokens(SemanticTokens {
      result_id: None,
      data: Vec::new(),
    });
  };

  let mut data: Vec<SemanticToken> = Vec::new();
  let mut last_line = 0u32;
  let mut last_start = 0u32;
  for token in &tokens {
    let Some(kind) = token_index(&token.kind) else {
      continue;
    };
    let start = to_position(source, token.span.0);
    let end = to_position(source, token.span.1);
    // A token spanning a line break cannot be delta-encoded as one token, and
    // the protocol has no shape for it. Multi-line strings are the real case
    // — `examples/strings.sdx`'s block string — and they are skipped rather
    // than truncated, because a token whose length ran past the end of its
    // line would shift every later token on that line.
    if end.line != start.line {
      continue;
    }
    let delta_line = start.line - last_line;
    let delta_start = if delta_line == 0 {
      start.character - last_start
    } else {
      start.character
    };
    data.push(SemanticToken {
      delta_line,
      delta_start,
      length: end.character - start.character,
      token_type: kind,
      token_modifiers_bitset: 0,
    });
    last_line = start.line;
    last_start = start.character;
  }

  SemanticTokensResult::Tokens(SemanticTokens {
    result_id: None,
    data,
  })
}
