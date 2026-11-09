use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::class_member::class_member;
use crate::parse::parser::method_context::MethodContext;
use redox_ast::ClassMember;
use redox_lexer::TokenKind;

/// The indented body of a `class` or `instance` — doc 18 §2.5.
///
/// One body parser for both, because the member grammar is the same one: an
/// associated type, an associated constant, or a method with or without a
/// body. Rust distinguishes what may appear in a `trait` from what may appear
/// in an `impl`; Redox does not, and leaves the distinction to rustc, which
/// is the same division of labour AGENTS.md states for every other check.
///
/// An empty body is legal — `class Marker t` with no members is a marker
/// trait, and Rust writes it `trait Marker {}`.
pub(crate) fn class_body(
  p: &mut Parser,
  ctx: &MethodContext,
) -> Result<Vec<ClassMember>, ParseError> {
  let mut members = Vec::new();

  let opens =
    p.check(&TokenKind::Newline) && p.peek_at(1).is_some_and(|t| t.kind == TokenKind::Indent);
  if !opens {
    return Ok(members);
  }
  p.advance();
  p.advance();

  loop {
    p.skip_separators();
    if p.check(&TokenKind::Dedent) || p.at_eof() {
      break;
    }
    members.push(class_member(p, ctx)?);
  }
  p.expect(&TokenKind::Dedent, "expected the body to end")?;
  Ok(members)
}
