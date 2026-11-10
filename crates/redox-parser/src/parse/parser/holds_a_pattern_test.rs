use crate::parse::parser::Parser;
use redox_lexer::{Keyword, TokenKind};

/// Whether a pattern test's `=` appears at depth zero before the scan's end.
///
/// With `one_operand`, the scan stops at `&&` and `||` — the question is
/// whether *this* operand is a pattern test. Without it, the scan runs to the
/// end of the condition — a line break, `else` for a `guard`, `->` for an arm
/// guard, or a closing delimiter never opened here.
///
/// `then` was in that list until plan 55 leaf 7 retired the keyword.
///
/// **Depth matters**, and more than it did when this looked for `<-`. A
/// record literal's `=` is inside its braces — `if p { x = 1 }.valid` is an
/// ordinary expression condition and must stay one — so counting brackets is
/// what keeps the two apart, and is the whole reason this is a function rather
/// than a `peek_at(1)` check.
///
/// `==` is a different token, and `+=` and friends are five more, so none of
/// them is seen here.
pub(crate) fn holds_a_pattern_test(p: &Parser, one_operand: bool) -> bool {
  let mut depth = 0i32;
  let mut i = 0;
  // A missing token is the end of input, which ends the condition too.
  while let Some(tok) = p.peek_at(i) {
    match &tok.kind {
      TokenKind::Eq if depth == 0 => return true,
      TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
      TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
        depth -= 1;
        // A closing delimiter never opened here ends the condition: it
        // belongs to something enclosing, so there is no `<-` left to find.
        if depth < 0 {
          return false;
        }
      }
      TokenKind::AndAnd | TokenKind::OrOr if depth == 0 && one_operand => return false,
      TokenKind::Kw(Keyword::Else) if depth == 0 => return false,
      // The arm-guard caller's terminator.
      TokenKind::Arrow if depth == 0 => return false,
      TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof => return false,
      _ => {}
    }
    i += 1;
  }
  false
}
