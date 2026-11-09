use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::constructor::constructor;
use crate::parse::parser::deriving::deriving;
use redox_ast::{DataDecl, Visibility};
use redox_lexer::{Keyword, TokenKind, Trivia};

/// Parse doc 18 §2.4's `data` or `newtype` declaration.
///
/// ```text
/// data Point = Point { x :: f64, y :: f64 }
///   deriving (Debug, Clone)
///
/// data Event
///   = Click { x :: i32, y :: i32 }
///   | Key char
///   | Close
///   deriving (Debug)
///
/// newtype Meters = Meters f64
/// data Marker = Marker
/// ```
///
/// Two layouts, one grammar. A short declaration sits on its line; a long one
/// puts each constructor on an indented line, the first after `=` and the
/// rest after `|`; and either may carry an indented `deriving` clause. The
/// parser does not distinguish them beyond the layout tokens, because they
/// are the same declaration — the same latitude a clause body has (§2.3), and
/// for the same reason: a reader chooses the layout, not a different
/// construct.
///
/// Written as **"skip layout, then look"** rather than as a lookahead over
/// `Newline` + `Indent`: the first draft decided the layout up front, and
/// `data Point = … ⏎ deriving (…)` fell between its two cases, because the
/// newline before `deriving` had already been consumed as a separator by the
/// time the lookahead ran.
pub(crate) fn data_decl(
  p: &mut Parser,
  leading: Vec<Trivia>,
  start: usize,
  visibility: Visibility,
) -> Result<DataDecl, ParseError> {
  let is_newtype = p.check(&TokenKind::Kw(Keyword::Newtype));
  p.advance();
  let name = p.ident("expected a type name after `data`")?;

  let mut constructors = Vec::new();
  let mut derives = Vec::new();
  let mut depth = 0usize;

  // The `=` row, the `|` rows and the `deriving` clause may each sit on the
  // declaration's own line or on an indented one, in any combination.
  loop {
    // Inside an indented block a line break only separates rows. *Outside*
    // one it terminates the declaration, and must be left for the caller —
    // consuming it here is what made `newtype Meters = Meters f64` report
    // "expected a line break after the declaration" at the *next* item.
    if depth > 0 {
      p.skip_separators();
    }

    let opens_a_block =
      p.check(&TokenKind::Newline) && p.peek_at(1).is_some_and(|t| t.kind == TokenKind::Indent);
    if opens_a_block {
      p.advance();
      p.advance();
      depth += 1;
      continue;
    }

    if p.check(&TokenKind::Eq) || p.check(&TokenKind::Or) {
      p.advance();
      constructors.push(constructor(p)?);
      continue;
    }

    if p.check(&TokenKind::Kw(Keyword::Deriving)) {
      derives = deriving(p)?;
      continue;
    }

    break;
  }

  for _ in 0..depth {
    p.skip_separators();
    p.expect(&TokenKind::Dedent, "expected the declaration to end")?;
  }

  Ok(DataDecl {
    name,
    constructors,
    derives,
    is_newtype,
    visibility,
    span: (start, p.prev_span().1),
    leading,
  })
}
