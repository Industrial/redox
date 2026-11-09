use crate::parse::parser::Parser;
use redox_ast::BinaryOp;
use redox_lexer::TokenKind;

/// The compound-assignment operator at the cursor, consumed if present —
/// doc 01 §4.3, plan 20.
///
/// ```text
/// counter += 1       counter += 1;
/// total   *= 2   →   total *= 2;
/// ```
///
/// **The only assignment spelling the specifications define.** Doc 01 §4.3
/// shows `mut counter = 0` then `counter += 1`; doc 18 §2.5 uses it on a
/// field. Plain reassignment appears in neither, because at statement level
/// `x = v` is a binding — see `StmtKind::Assign` for why telling the two
/// apart is not Redox's to do.
///
/// `??=` is **not** here: doc 18 §7 retires it in favour of its expanded
/// form, so the token still lexes and nothing accepts it.
pub(crate) fn compound_assign(p: &mut Parser) -> Option<BinaryOp> {
  let op = match p.peek().kind {
    TokenKind::PlusEq => BinaryOp::Add,
    TokenKind::MinusEq => BinaryOp::Sub,
    TokenKind::StarEq => BinaryOp::Mul,
    TokenKind::SlashEq => BinaryOp::Div,
    TokenKind::PercentEq => BinaryOp::Rem,
    _ => return None,
  };
  p.advance();
  Some(op)
}
