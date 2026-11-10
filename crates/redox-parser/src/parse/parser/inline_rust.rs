use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::rust_hatch::rust_hatch;
use redox_ast::InlineRust;
use redox_lexer::{Keyword, TokenKind, Trivia};

/// `rust """…"""` — doc 01 §23.1's inline Rust escape hatch, plan 45 leaf 1.
///
/// The block's body is taken **verbatim**, through
/// [`redox_lexer::verbatim_triple`] rather than through the token's decoded
/// parts. That is not a refinement: the lexer decodes escapes, so a hatch
/// carrying `println!("a\nb")` would reach the emitter holding a real newline
/// inside a Rust string literal and produce code that does not compile.
///
/// Plan 64 leaf 4 moved the body-reading into [`rust_hatch`], which both this
/// and the expression position use: the layout-delimited spelling — `rust`
/// over an indented block — is read there too, and a second reader would be a
/// second place for the hatch's rules to drift.
pub(crate) fn inline_rust(
  p: &mut Parser,
  leading: Vec<Trivia>,
  start: usize,
) -> Result<InlineRust, ParseError> {
  p.expect(&TokenKind::Kw(Keyword::Rust), "expected `rust`")?;

  let (code, _) = rust_hatch(p)?;

  Ok(InlineRust {
    code,
    span: (start, p.prev_span().1),
    leading,
  })
}
