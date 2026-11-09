use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::pattern::pattern;
use redox_ast::Pattern;
use redox_lexer::TokenKind;

/// Parse an equation's pattern row: the patterns between the head and the `=`.
///
/// Doc 18 §2.3. `fib 0` binds nothing and tests for zero; `fib n` binds; and
/// `f _` matches and discards. Replaces `binders`, which accepted only names
/// and said in its own documentation that patterns were deferred.
///
/// Deliberately three shapes — a name, a literal, `_` — which is what doc 18
/// §2.3's examples use. The full pattern grammar is doc 04's plan 33 and needs
/// §2.4's `data` to be worth testing against; a pattern form nothing can
/// construct is code with no way to exercise it.
///
/// The row ends at `=`, which opens a plain body, or at a line break, which
/// opens a guard ladder whose `|` arrives indented on the next line. Not at
/// the bar: the bare bar is `TokenKind::Or` — `Pipe` is `|>` — and testing for
/// it instead of the line break left `tier spend` trying to read the break as
/// a pattern.
pub(crate) fn patterns(p: &mut Parser) -> Result<Vec<Pattern>, ParseError> {
  let mut found = Vec::new();
  while !p.at_eof() && !p.check(&TokenKind::Eq) && !p.check(&TokenKind::Newline) {
    found.push(pattern(p)?);
  }
  Ok(found)
}
