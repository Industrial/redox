use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::capture_type::capture_type;
use crate::parse::parser::type_stop::TypeStop;
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
  let name = p.ident("expected a field name")?;
  p.expect(&TokenKind::ColonColon, "expected `::` after the field name")?;
  let ty = capture_type(p, "expected a field type", TypeStop::Greedy)?;
  Ok(FieldDecl {
    name,
    ty,
    span: (start, p.prev_span().1),
  })
}
