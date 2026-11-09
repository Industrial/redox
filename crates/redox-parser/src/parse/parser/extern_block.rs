use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::parse_spine::parse_spine;
use crate::parse::parser::signature_only::signature_only;
use redox_ast::{ExternBlock, ForeignFn};
use redox_lexer::{Keyword, TokenKind, Trivia};

/// `extern "C"` and its indented declarations — plan 45 leaf 3.
///
/// ```text
/// extern "C"
///   abs :: i32 -> i32
/// ```
///
/// Doc 01 §26 names FFI as a requirement and doc 03 §23 item 2 rules out the
/// escape hatch for it, so this is the one leaf of plan 45 the specification
/// asks for by name.
///
/// The declarations are **signatures with no equation**, which is doc 18
/// §2.5's required method exactly — so `signature_only` is reused rather than
/// reimplemented, and a foreign parameter gets the generated name doc 02
/// reserves for a row that has none.
///
/// The ABI is any string literal. Redox keeps no list of ABIs and checks
/// nothing about it: an ABI Rust does not know is rustc's to reject, which is
/// AGENTS.md's every-check-rustc-makes rule and the reason this function is as
/// short as it is.
pub(crate) fn extern_block(
  p: &mut Parser,
  leading: Vec<Trivia>,
  start: usize,
) -> Result<ExternBlock, ParseError> {
  p.expect(&TokenKind::Kw(Keyword::Extern), "expected `extern`")?;

  let abi = match &p.peek().kind {
    TokenKind::Str { raw, .. } => raw.clone(),
    _ => {
      return Err(ParseError::expected(
        "expected an ABI string after `extern`, as in `extern \"C\"`",
        p.peek_span(),
      ));
    }
  };
  p.advance();
  p.newline_with_trailing("expected a line break after the ABI")?;

  let mut decls = Vec::new();
  p.expect(
    &TokenKind::Indent,
    "expected indented `extern` declarations",
  )?;
  loop {
    p.skip_separators();
    if p.check(&TokenKind::Dedent) || p.at_eof() {
      break;
    }
    let decl_leading = p.peek_leading();
    let decl_start = p.peek_span().0;
    let name = p.ident("expected a foreign function's name")?;
    p.expect(
      &TokenKind::ColonColon,
      "expected `::` after the foreign function's name",
    )?;
    let spine = parse_spine(p)?;
    let span = (decl_start, p.prev_span().1);
    decls.push(ForeignFn {
      name,
      sig: signature_only(&spine, span),
      span,
      leading: decl_leading,
    });
    p.newline_with_trailing("expected a line break after the declaration")?;
  }
  p.expect(&TokenKind::Dedent, "expected the `extern` block to end")?;

  Ok(ExternBlock {
    abi,
    decls,
    span: (start, p.prev_span().1),
    leading,
  })
}
