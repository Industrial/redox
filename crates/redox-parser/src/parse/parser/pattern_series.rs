use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::pattern_alt::pattern_alt;
use redox_ast::Pattern;
use redox_lexer::TokenKind;

/// A comma-separated run of patterns, up to but not including `close` — plan
/// 33.
///
/// Shared by the tuple and slice forms and by a record pattern's field list,
/// because all three are the same shape: patterns, commas, a trailing comma
/// permitted, and a closing delimiter the caller consumes.
///
/// Each element is a full [`pattern_alt`], so `[Some 0 or None, b]` and
/// `(rest @ .., x)` work without this knowing about either — the vocabulary is
/// shared, which is plan 22 §10.1's whole point about one `Pattern` enum.
pub(crate) fn pattern_series(
  p: &mut Parser,
  close: &TokenKind,
) -> Result<Vec<Pattern>, ParseError> {
  let mut items = Vec::new();
  // Plan 65 leaf 1: the bracket this sits inside opens a layout context, so
  // its lines carry layout tokens the series itself has no use for.
  p.skip_layout();
  while !p.check(close) {
    items.push(pattern_alt(p)?);
    p.skip_layout();
    if !p.check(&TokenKind::Comma) {
      break;
    }
    p.advance();
    p.skip_layout();
  }
  Ok(items)
}
