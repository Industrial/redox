use crate::parse::parser::Parser;
use redox_ast::UnaryOp;
use redox_lexer::{Keyword, TokenKind};

/// A unary prefix operator at the cursor, consumed if present.
///
/// `&mut` is two tokens, so `&` is consumed first and `mut` probed after —
/// which is why this returns rather than peeking.
pub(crate) fn unary_op(p: &mut Parser) -> Option<UnaryOp> {
  let op = match p.peek().kind {
    TokenKind::Minus => UnaryOp::Neg,
    TokenKind::Bang => UnaryOp::Not,
    TokenKind::Star => UnaryOp::Deref,
    TokenKind::Amp => {
      p.advance();
      return Some(if p.eat_keyword(Keyword::Mut) {
        UnaryOp::RefMut
      } else {
        UnaryOp::Ref
      });
    }
    _ => return None,
  };
  p.advance();
  Some(op)
}
