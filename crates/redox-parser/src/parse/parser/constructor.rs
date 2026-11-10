use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::constructor_fields::constructor_fields;
use redox_ast::Constructor;

/// One constructor of a `data` declaration — doc 18 §2.4.
///
/// `Click { x : i32, y : i32 }`, `Key char`, `Close`. The name is an
/// ordinary identifier here rather than a checked-uppercase one: doc 18 §5.3
/// makes case *meaningful* — constructors and types begin uppercase, values
/// and type variables lowercase — but enforcing it is a diagnostic, and a
/// diagnostic with no code in plan 05's catalogue would fail
/// `code_coverage.rs`. The rule is used where it decides a reading, not as a
/// gate.
/// `leading` is the caller's, and has to be: a constructor's doc comment
/// sits above the `=` or `|` that opens its line, so `data_decl` reads it
/// before eating the separator and passes it in. Reading it here would look
/// one token too late and find nothing — which is what plan 68's first
/// attempt did.
pub(crate) fn constructor(
  p: &mut Parser,
  leading: Vec<redox_lexer::Trivia>,
) -> Result<Constructor, ParseError> {
  let start = p.peek_span().0;
  let name = p.ident("expected a constructor name")?;
  let fields = constructor_fields(p)?;
  Ok(Constructor {
    name,
    fields,
    leading,
    span: (start, p.prev_span().1),
  })
}
