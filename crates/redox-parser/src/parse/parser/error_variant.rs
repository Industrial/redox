use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::capture_type::capture_type;
use crate::parse::parser::starts_a_field_type::starts_a_field_type;
use crate::parse::parser::type_stop::TypeStop;
use redox_ast::ErrorVariant;
use redox_lexer::{Keyword, TokenKind};

/// One row of an `errors` declaration — doc 03 §12.5.
///
/// ```text
/// Missing String        "missing key: #{0}"
/// Invalid String u32    "invalid value #{0} at line #{1}"
/// Closed                "closed"
/// from io::Error as Io  "io error: #{0}"
/// ```
///
/// Two shapes. `from T as Name` names the type first and the variant second,
/// which is the order it reads in — *from* this, call it that — and it is
/// what asks for the extra `From` implementation. Everything else is a
/// variant name followed by its juxtaposed payload types (doc 18 §2.1).
///
/// The message terminates the field list, needing no separator: a string
/// literal cannot begin a type. That is what lets the rows line up as a
/// table, which is how doc 03 §12.5 and doc 18 §9 both write them.
pub(crate) fn error_variant(p: &mut Parser) -> Result<ErrorVariant, ParseError> {
  let start = p.peek_span().0;

  // `from` is not a reserved word — doc 18 §7's table does not add one — so
  // it is matched as the identifier it is. A variant genuinely named `from`
  // would be lowercase, which doc 18 §5.3 reserves for values, so the
  // collision is already ruled out by the capitalisation rule.
  let is_from = matches!(&p.peek().kind, TokenKind::Ident(text) if text == "from");

  let (name, fields, from) = if is_from {
    p.advance();
    let ty = capture_type(p, "expected a type after `from`", TypeStop::Greedy)?;
    // `as` is a reserved word, so it is matched as the keyword it is.
    p.expect(
      &TokenKind::Kw(Keyword::As),
      "expected `as` between the type and the variant's name",
    )?;
    let name = p.ident("expected the variant's name after `as`")?;
    (name, vec![ty], true)
  } else {
    let name = p.ident("expected a variant name")?;
    let mut fields = Vec::new();
    while starts_a_field_type(&p.peek().kind) {
      fields.push(capture_type(p, "expected a payload type", TypeStop::Atom)?);
    }
    (name, fields, false)
  };

  let message = match &p.peek().kind {
    TokenKind::Str { raw, .. } => raw.clone(),
    _ => {
      return Err(ParseError::expected(
        "expected the variant's `Display` message, as a string literal",
        p.peek_span(),
      ));
    }
  };
  p.advance();
  p.newline_with_trailing("expected a line break after the variant")?;

  Ok(ErrorVariant {
    name,
    fields,
    from,
    message,
    span: (start, p.prev_span().1),
  })
}
