use crate::parse::parser::Parser;
use redox_lexer::TokenKind;

/// Whether a `<-` appears at depth zero before this statement ends.
///
/// A `<-` inside brackets belongs to something nested and is not this
/// statement's, so the depth count is what keeps a `[ … ]` or a `( … )` from
/// claiming one.
pub(crate) fn reaches_a_draw_arrow(p: &Parser) -> bool {
  let mut depth = 0i32;
  let mut i = 0;
  while let Some(tok) = p.peek_at(i) {
    match &tok.kind {
      TokenKind::DrawArrow if depth == 0 => return true,
      TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
      TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
        depth -= 1;
        if depth < 0 {
          return false;
        }
      }
      // An `=` at depth zero means this is a binding, not an await-bind, and
      // the binding parser gets it. Stopping here rather than at the end of
      // the line is what keeps `x = f y <- z` from being read as either.
      TokenKind::Eq if depth == 0 => return false,
      TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof => return false,
      _ => {}
    }
    i += 1;
  }
  false
}
