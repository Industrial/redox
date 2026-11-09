use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::capture_type::capture_type;
use crate::parse::parser::type_stop::TypeStop;
use redox_ast::TypeAlias;
use redox_lexer::{Keyword, TokenKind, Trivia};

/// `type Handler = Box<dyn Fn(Req) -> Res + Send>` — doc 18 §2.4.
///
/// The one declaration form that is already Rust: the keyword, the `=` and
/// the type are unchanged. The type is captured verbatim, as every type in
/// Redox is (doc 03 §14.1) — including the `->` inside `Fn(…) -> …`, which
/// `capture_type` handles because an `->` after a `)` does not end a type.
/// That refinement was written for parameter types in plan 03, and it is what
/// makes doc 18 §2.4's own alias example reachable without a second rule.
pub(crate) fn type_alias(
  p: &mut Parser,
  leading: Vec<Trivia>,
  start: usize,
) -> Result<TypeAlias, ParseError> {
  p.expect(&TokenKind::Kw(Keyword::Type), "expected `type`")?;
  let name = p.ident("expected a name after `type`")?;
  p.expect(&TokenKind::Eq, "expected `=` after the alias's name")?;
  let ty = capture_type(p, "expected a type", TypeStop::Greedy)?;
  Ok(TypeAlias {
    name,
    ty,
    span: (start, p.prev_span().1),
    leading,
  })
}
