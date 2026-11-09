use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::Expr;
use redox_diag::Code;
use redox_lexer::{Keyword, TokenKind};

/// Parse doc 03 §7.2's `do` block — plan 20.
///
/// ```text
/// found = do 'search
///   for c in candidates
///     if c.matches q
///       break 'search Some c
///   None
/// ```
///
/// An **expression**, because §7.2's own example assigns one to a binding.
/// The optional label makes it Rust's labelled block — native since 1.65 —
/// which is what *"permitting early exit with a value"* needs.
pub(crate) fn do_block(p: &mut Parser) -> Result<Expr, ParseError> {
  let start = p.peek_span().0;
  p.expect(&TokenKind::Kw(Keyword::Do), "expected `do`")?;

  let label = match &p.peek().kind {
    TokenKind::Label(name) => {
      let name = name.clone();
      p.advance();
      Some(name)
    }
    _ => None,
  };

  p.newline_with_trailing("expected a line break after `do`")?;
  let body = p.indented_block(Code::EmptyBody, "expected an indented `do` body")?;

  Ok(Expr::Do {
    label,
    body,
    span: (start, p.prev_span().1),
  })
}
