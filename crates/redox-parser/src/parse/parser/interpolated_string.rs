use crate::ParseError;
use crate::parse::parser::expr_from_tokens::expr_from_tokens;
use crate::parse::parser::rust_string_body::rust_string_body;
use redox_ast::{Expr, StrInterpPart};
use redox_lexer::{Span, StrPart};

/// Build an [`Expr::StrInterp`] from the lexer's parts — doc 03 §11.1, plan 23.
///
/// Plan 02's lexer finished the scanning twenty-one plans ago: each `#{…}` is
/// already found, its body already lexed with the ordinary scanners, and its
/// format spec already split off. `StrPart`'s own doc comment says `parts` is
/// *"what plan 23's emitter consumes"*. This is where it finally does.
///
/// # The body goes through the ordinary expression parser
///
/// Decision 2. The lexer hands over a `Vec<Token>` with spans that index the
/// real file, so the seam is a token slice and the grammar on the far side is
/// the same one. Writing a small dedicated grammar for interpolations is how
/// two grammars drift apart, and §8 records Emerald accepting only an
/// identifier here for eleven plans.
pub(crate) fn interpolated_string(parts: &[StrPart], span: Span) -> Result<Expr, ParseError> {
  let mut out = Vec::with_capacity(parts.len());
  for part in parts {
    match part {
      // Escaped **here**, not in the emitter. The lexer decodes escapes, so
      // the text has to be re-escaped for Rust somewhere, and doing it in the
      // parser means one implementation serves the `"""` block, the byte and
      // C-string literals and this — rather than the same function living in
      // two crates that cannot call each other.
      StrPart::Text(text) => out.push(StrInterpPart::Text(rust_string_body(text))),
      StrPart::Interp { tokens, spec } => out.push(StrInterpPart::Interp {
        // The first bad interpolation ends the string, as the first bad
        // anything ends any other expression. The parser's batch recovery
        // happens at item boundaries, which is where it belongs.
        expr: expr_from_tokens(tokens, span)?,
        spec: spec.clone(),
      }),
    }
  }
  Ok(Expr::StrInterp { parts: out, span })
}
