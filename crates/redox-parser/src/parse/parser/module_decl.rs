use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::{ModuleDecl, Visibility};
use redox_lexer::{Keyword, TokenKind, Trivia};

/// Parse doc 18 §7's `module` — plan 14 §1.2.
///
/// ```text
/// module config
/// export module parser
///
/// module tests
///   test "it parses"
///     assert! \-> 1 == 1
/// ```
///
/// A name, and an optional indented body of items. The body is **recursive**:
/// a `module` inside a `module` is the same declaration, and nothing stops
/// it, because nothing in Rust does either.
///
/// Doc 18 §8 declines Haskell's module header, so this is `mod` and not a
/// file-level declaration — see [`ModuleDecl`] for the whole of that
/// argument.
pub(crate) fn module_decl(
  p: &mut Parser,
  leading: Vec<Trivia>,
  start: usize,
  visibility: Visibility,
) -> Result<ModuleDecl, ParseError> {
  p.expect(&TokenKind::Kw(Keyword::Module), "expected `module`")?;
  let name = p.ident("expected the module's name")?;

  // A body is a line break then an indent. Without one this is `mod name;`,
  // which loads a sibling file — a different declaration, not an empty one,
  // which is why `items` is an `Option`.
  let has_body =
    p.check(&TokenKind::Newline) && p.peek_at(1).is_some_and(|t| t.kind == TokenKind::Indent);
  if !has_body {
    return Ok(ModuleDecl {
      name,
      items: None,
      visibility,
      span: (start, p.prev_span().1),
      leading,
    });
  }

  p.advance();
  p.advance();
  let items = p.items_until_dedent()?;
  Ok(ModuleDecl {
    name,
    items: Some(items),
    visibility,
    span: (start, p.prev_span().1),
    leading,
  })
}
