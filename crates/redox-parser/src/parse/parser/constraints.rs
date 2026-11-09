use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::Constraint;
use redox_lexer::TokenKind;

/// The `Trait t, Other u =>` clause, if one is here — doc 18 §2.5.
///
/// `=>` means a constraint and nothing else, which is exactly what doc 18 §7
/// retires it as an arm arrow for. Recognising it needs **arbitrary
/// lookahead**, because `class Shape t` and `class Debug t => Shape t` begin
/// identically: the constraints are read speculatively and the parser rewinds
/// if no `=>` arrives.
///
/// Rewinding rather than restructuring the grammar is the cheap answer here
/// and stays cheap: the span scanned is one line of names, and the
/// alternative — reading a name list and deciding afterwards what it was —
/// would put the ambiguity in two places instead of one.
pub(crate) fn constraints(p: &mut Parser) -> Result<Vec<Constraint>, ParseError> {
  let mark = p.position();
  let mut found = Vec::new();

  // Haskell parenthesises a list of two or more; one needs no parentheses.
  let parenthesised = p.check(&TokenKind::LParen);
  if parenthesised {
    p.advance();
  }

  loop {
    let start = p.peek_span().0;
    let Some(trait_name) = p.try_ident() else {
      break;
    };
    let Some(var) = p.try_ident() else { break };
    found.push(Constraint {
      trait_name,
      var,
      span: (start, p.prev_span().1),
    });
    if !p.check(&TokenKind::Comma) {
      break;
    }
    p.advance();
  }

  if parenthesised && p.check(&TokenKind::RParen) {
    p.advance();
  }

  if found.is_empty() || !p.check(&TokenKind::FatArrow) {
    p.rewind(mark);
    return Ok(Vec::new());
  }
  p.advance();
  Ok(found)
}
