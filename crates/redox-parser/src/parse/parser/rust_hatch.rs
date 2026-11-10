use crate::ParseError;
use crate::parse::parser::Parser;
use redox_lexer::{Span, TokenKind, verbatim_triple};

/// The body of a `rust` escape hatch, in either spelling — plan 64 leaf 4.
///
/// ```text
/// rust """              markup = rust
///   let x = 1;            html! { <div/> }
///   """
/// ```
///
/// One function for both because the two spellings differ only in where the
/// lexer stopped reading Redox: the `"""` form is a `Str` token whose raw
/// text is sliced by [`verbatim_triple`], and the layout form arrives already
/// dedented as a [`TokenKind::RustBlock`]. Everything after that is identical,
/// so a second reader would be a second place for the hatch's rules to drift.
///
/// The cursor is expected **on the body**, with `rust` already consumed.
pub(crate) fn rust_hatch(p: &mut Parser) -> Result<(String, Span), ParseError> {
  let span = p.peek_span();
  if let TokenKind::RustBlock(code) = &p.peek().kind {
    let code = code.clone();
    p.advance();
    return Ok((code, span));
  }

  let TokenKind::Str { raw, .. } = &p.peek().kind else {
    return Err(ParseError::expected(
      "expected a `\"\"\"` block of Rust after `rust`, or an indented block \
       on the lines under it",
      span,
    ));
  };
  let raw = raw.clone();
  p.advance();

  // A single-quoted string is **rejected** rather than accepted as a one-line
  // hatch: doc 01 §23.1 writes `"""`, a one-line escape hatch is a worse
  // thing to encourage than a two-line one, and — the deciding reason — a
  // `"…"` literal has nowhere to put the Rust code's own quotes.
  verbatim_triple(&raw)
    .map(|code| (code, span))
    .ok_or_else(|| {
      ParseError::expected(
        "inline Rust needs a `\"\"\"` block or an indented one: a single-quoted \
       string cannot hold Rust's own quotes",
        span,
      )
    })
}
