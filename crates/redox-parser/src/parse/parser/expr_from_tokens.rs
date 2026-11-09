use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::parse_expr::parse_expr;
use redox_ast::Expr;
use redox_lexer::{Span, Token, TokenKind};

/// Parse one expression from a detached token slice.
///
/// `lex_fragment` strips structural tokens **and** `Eof`, so the slice needs a
/// terminator before `Parser` runs over it: `peek` clamps to the last token
/// rather than reading past the end, and a clamp that is not `Eof` is a token
/// the grammar would try again. The synthetic `Eof` takes the string's span so
/// an error at the end of an interpolation points inside the string.
pub(crate) fn expr_from_tokens(tokens: &[Token], span: Span) -> Result<Expr, ParseError> {
  let mut toks = tokens.to_vec();
  toks.push(Token {
    kind: TokenKind::Eof,
    span,
    ws_before: false,
    leading: Vec::new(),
    trailing: None,
  });

  let mut p = Parser::new(&toks);
  let expr = parse_expr(&mut p, 0)?;
  // An interpolation holds **one** expression. Leftover tokens mean the body
  // was something else — `#{a b c}` with no application, say — and silently
  // dropping them would emit a format argument the author did not write.
  if !p.at_eof() {
    return Err(ParseError::with(
      redox_diag::Code::ExpectedToken,
      "an interpolation holds one expression; `}` was expected here",
      p.peek_span(),
    ));
  }
  Ok(expr)
}
