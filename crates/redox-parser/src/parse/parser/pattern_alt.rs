use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::pattern::pattern;
use redox_ast::Pattern;
use redox_lexer::{Keyword, TokenKind};

/// One pattern, or several joined by `or`.
///
/// Doc 03's alternation combinator, kept by doc 18 §5.4 **in order to keep
/// the guard rule unambiguous**: alternation spelled `or` leaves `|` free to
/// mean the guard everywhere a pattern can be followed by one. Rust's own
/// spelling would have cost that, which is the whole argument for not simply
/// adopting it.
///
/// Lowers to Rust's `|`, unchanged — so this is a spelling, not a lowering.
pub(crate) fn pattern_alt(p: &mut Parser) -> Result<Pattern, ParseError> {
  let first = pattern(p)?;
  if !p.check(&TokenKind::Kw(Keyword::Or)) {
    return Ok(first);
  }

  let mut alts = vec![first];
  while p.eat_keyword(Keyword::Or) {
    alts.push(pattern(p)?);
  }
  Ok(Pattern::Or(alts))
}
