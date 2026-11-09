use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::InlineRust;
use redox_lexer::{Keyword, TokenKind, Trivia, verbatim_triple};

/// `rust """…"""` — doc 01 §23.1's inline Rust escape hatch, plan 45 leaf 1.
///
/// The block's body is taken **verbatim**, through
/// [`redox_lexer::verbatim_triple`] rather than through the token's decoded
/// parts. That is not a refinement: the lexer decodes escapes, so a hatch
/// carrying `println!("a\nb")` would reach the emitter holding a real newline
/// inside a Rust string literal and produce code that does not compile.
///
/// A single-quoted string is **rejected** rather than accepted as a one-line
/// hatch. Doc 01 §23.1 writes `"""`, a one-line escape hatch is a worse thing
/// to encourage than a two-line one, and — the deciding reason — a `"…"`
/// literal has nowhere to put the Rust code's own quotes.
pub(crate) fn inline_rust(
  p: &mut Parser,
  leading: Vec<Trivia>,
  start: usize,
) -> Result<InlineRust, ParseError> {
  p.expect(&TokenKind::Kw(Keyword::Rust), "expected `rust`")?;

  let raw = match &p.peek().kind {
    TokenKind::Str { raw, .. } => raw.clone(),
    _ => {
      return Err(ParseError::expected(
        "expected a `\"\"\"` block of Rust after `rust`",
        p.peek_span(),
      ));
    }
  };
  let span = p.peek_span();
  p.advance();

  let Some(code) = verbatim_triple(&raw) else {
    return Err(ParseError::expected(
      "inline Rust needs a `\"\"\"` block: a single-quoted string cannot hold \
       Rust's own quotes",
      span,
    ));
  };

  Ok(InlineRust {
    code,
    span: (start, p.prev_span().1),
    leading,
  })
}
