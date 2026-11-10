use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::pattern_alt::pattern_alt;
use redox_ast::{Pattern, RecordPatternField};
use redox_lexer::TokenKind;

/// `Point { x = 0, y, .. }` — doc 03 §6.3's named record pattern, plan 33.
///
/// The cursor is on the `{`; `path` is the head already read.
///
/// # `=` and not `:`
///
/// §6.3 published `Point{x: 0, y}`. Doc 18 §2.4 changed the record *literal*
/// to `=` — `Point { x = 10, y = 20 }` — and plan 52 made `:` the type marker
/// everywhere, which leaves it no job supplying a *value* inside braces. So
/// the pattern uses `=` and reads identically to the literal that builds it,
/// lowering to Rust's `Point { x: 0, y }`.
///
/// Two spellings of one shape differing only by position is exactly what doc
/// 18 §5.3 exists to remove.
///
/// # The anonymous form is deleted, not missing
///
/// §6.3's second spelling — `{active: true, age: > 18}` with no path — is
/// deleted by doc 18 §2.3 along with the relational patterns it was built on,
/// and `RDX0042` and `RDX0043` go with it. It lowered to
/// `u.active && u.age > 18`, which is *shorter written directly*.
pub(crate) fn record_pattern(
  p: &mut Parser,
  head: String,
  start: usize,
) -> Result<Pattern, ParseError> {
  // A `::`-joined path, for `Event::Click { x }`. `::` is a path separator and
  // nothing else — plan 52 made `:` the type marker.
  let mut path = head;
  while p.check(&TokenKind::ColonColon) {
    p.advance();
    path.push_str("::");
    path.push_str(&p.ident("expected a name after `::`")?.text);
  }

  p.expect(&TokenKind::LBrace, "expected `{` to open a record pattern")?;

  let mut fields = Vec::new();
  let mut rest = false;
  while !p.check(&TokenKind::RBrace) {
    // `..` ends the field list — *"and other fields"*. Rust requires it last,
    // so nothing follows it here either.
    if p.check(&TokenKind::DotDot) {
      p.advance();
      rest = true;
      break;
    }

    let name = p.ident("expected a field name in a record pattern")?;
    // No `=` is doc 03 §6.3's *"field pun"*: the field binds under its own
    // name, which is Rust's own shorthand and needs no spelling.
    let pattern = if p.check(&TokenKind::Eq) {
      p.advance();
      Some(pattern_alt(p)?)
    } else {
      None
    };
    fields.push(RecordPatternField { name, pattern });

    if !p.check(&TokenKind::Comma) {
      break;
    }
    p.advance();
  }

  p.expect(
    &TokenKind::RBrace,
    "expected `}` to close the record pattern",
  )?;

  Ok(Pattern::Record {
    path,
    fields,
    rest,
    span: (start, p.prev_span().1),
  })
}
