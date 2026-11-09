use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::capture_type::capture_type;
use crate::parse::parser::type_stop::TypeStop;
use crate::parse::parser::visibility::visibility;
use redox_ast::FieldDecl;
use redox_lexer::TokenKind;

/// One `name :: Type` field of a record constructor — doc 18 §2.4.
///
/// The same `::` as a signature line, a lambda binder and an ascription
/// (§5.2). One notation for "this has this type", in every position — which
/// is the argument for `::` over a second sigil, and the reason records use
/// `=` for *values*: `::` introduces a type, `=` introduces a value, in a
/// declaration and a literal alike.
pub(crate) fn field_decl(p: &mut Parser) -> Result<FieldDecl, ParseError> {
  let start = p.peek_span().0;
  // Plan 48 leaf 2. Read before the visibility marker, as an item's trivia
  // is: a `--- doc` on a field attaches to the first token of the field, and
  // that is the marker when there is one.
  let leading = p.peek_leading();
  // §9.6: "Per-field and per-variant visibility use the same markers inside
  // a declaration body."
  let visibility = visibility(p)?;
  let name = p.ident("expected a field name")?;
  p.expect(&TokenKind::ColonColon, "expected `::` after the field name")?;
  let ty = capture_type(p, "expected a field type", TypeStop::Greedy)?;
  Ok(FieldDecl {
    name,
    ty,
    visibility,
    leading,
    span: (start, p.prev_span().1),
  })
}
