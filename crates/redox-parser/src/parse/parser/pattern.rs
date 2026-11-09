use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::range_or_literal::range_or_literal;
use crate::parse::parser::token_text::token_text;
use redox_ast::Pattern;
use redox_lexer::{Keyword, TokenKind};

/// One pattern in an equation's row.
///
/// Doc 18 §2.3, and a sibling of [`super::patterns`] rather than a private
/// helper inside it, because doc 06 §1 permits one construct per file.
pub(crate) fn pattern(p: &mut Parser) -> Result<Pattern, ParseError> {
  let span = p.peek_span();
  match &p.peek().kind {
    TokenKind::Ident(text) if text == "_" => {
      p.advance();
      Ok(Pattern::Wildcard(span))
    }
    TokenKind::Ident(_) => Ok(Pattern::Binding(p.ident("expected a pattern")?)),
    // A literal **tests**, and is carried as verbatim source text — the same
    // discipline doc 03 §14.1 applies to types, so Redox never has to agree
    // with rustc about what a literal means.
    kind @ (TokenKind::Int(_)
    | TokenKind::Float(_)
    | TokenKind::Char(_)
    | TokenKind::Byte(_)
    | TokenKind::Kw(Keyword::True)
    | TokenKind::Kw(Keyword::False)) => {
      let text = token_text(kind);
      p.advance();
      Ok(range_or_literal(p, text, span))
    }
    TokenKind::Str { raw, .. } => {
      let text = raw.clone();
      p.advance();
      Ok(Pattern::Literal { text, span })
    }
    _ => Err(ParseError::with(
      redox_diag::Code::ExpectedName,
      "expected a parameter name, a literal pattern, a range, `_`, or `=`",
      span,
    )),
  }
}
