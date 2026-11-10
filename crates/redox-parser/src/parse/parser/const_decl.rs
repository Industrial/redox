use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::capture_type::capture_type;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::type_stop::TypeStop;
use redox_ast::{ConstDecl, Visibility};
use redox_lexer::{Keyword, TokenKind, Trivia};

/// Parse doc 03 §13.4's module-scope `const` or `static` — plan 16.
///
/// ```text
/// const MAX : usize = 1024
/// static COUNT : u32 = 0
/// export const LIMIT : usize = 8
/// ```
///
/// The value is **required**. A class's associated constant may be declared
/// without one — the instance supplies it — but a module-scope binding has
/// nowhere else for a value to come from, so asking here gives a better
/// message than rustc's.
pub(crate) fn const_decl(
  p: &mut Parser,
  leading: Vec<Trivia>,
  start: usize,
  visibility: Visibility,
) -> Result<ConstDecl, ParseError> {
  let is_static = p.check(&TokenKind::Kw(Keyword::Static));
  p.advance();

  let name = p.ident("expected a name after `const`")?;
  p.expect(
    &TokenKind::Colon,
    "expected `:` and a type — doc 03 §13.4 requires one on a module-scope binding",
  )?;
  let ty = capture_type(p, "expected the binding's type", TypeStop::Greedy)?;
  p.expect(
    &TokenKind::Eq,
    "expected `=` and a value — a module-scope binding must be initialised",
  )?;
  let value = parse_expr(p, 0)?;

  Ok(ConstDecl {
    name,
    ty,
    value,
    is_static,
    visibility,
    span: (start, p.prev_span().1),
    leading,
  })
}
