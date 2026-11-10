use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::capture_type::capture_type;
use crate::parse::parser::field_decl::field_decl;
use crate::parse::parser::starts_a_field_type::starts_a_field_type;
use crate::parse::parser::type_stop::TypeStop;
use redox_ast::ConstructorFields;
use redox_lexer::TokenKind;

/// What follows a constructor's name — doc 18 §2.4.
///
/// Braces open a record; a type-leading token opens a tuple; a line break,
/// `|` or a dedent means the constructor is a unit.
///
/// The tuple case reads its field types by **juxtaposition**, which is doc 18
/// §2.1's reading everywhere else — so `Key char` needs no notation of its
/// own, and `Pair i32 i32` falls out. `capture_type` already knows where one
/// type ends, which is the whole of what makes this work.
pub(crate) fn constructor_fields(p: &mut Parser) -> Result<ConstructorFields, ParseError> {
  if p.check(&TokenKind::LBrace) {
    p.advance();
    let mut fields = Vec::new();
    loop {
      p.skip_layout();
      if p.check(&TokenKind::RBrace) {
        break;
      }
      fields.push(field_decl(p)?);
      p.skip_layout();
      if !p.check(&TokenKind::Comma) {
        break;
      }
      p.advance();
    }
    p.skip_layout();
    p.expect(&TokenKind::RBrace, "expected `}` to close the record")?;
    return Ok(ConstructorFields::Record(fields));
  }

  let mut types = Vec::new();
  while starts_a_field_type(&p.peek().kind) {
    types.push(capture_type(p, "expected a field type", TypeStop::Atom)?);
  }
  if types.is_empty() {
    return Ok(ConstructorFields::Unit);
  }
  Ok(ConstructorFields::Tuple(types))
}
