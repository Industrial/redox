use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::constructor::constructor;
use crate::parse::parser::deriving::deriving;
use crate::parse::parser::generalise_data::generalise_data;
use crate::parse::parser::newtype_shape::newtype_shape;
use redox_ast::{DataDecl, Visibility};
use redox_lexer::{Keyword, TokenKind, Trivia};

/// Parse doc 18 §2.4's `data` or `newtype` declaration.
///
/// ```text
/// data Point = Point { x : f64, y : f64 }
///   deriving (Debug, Clone)
///
/// data Event
///   = Click { x : i32, y : i32 }
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
  // Plan 45 leaf 4. The third keyword this one function serves, and the
  // reason it reads the keyword rather than being told which it was.
  let is_union = p.check(&TokenKind::Kw(Keyword::Union));
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
      // **Before the separator is eaten**, which is plan 68's finding: a
      // constructor's doc comment goes on its own line above the variant, so
      // the trivia attaches to the `=` or `|` that opens that line and
      // `peek_leading` at the *name* looks one token too late.
      let leading = p.peek_leading();
      p.advance();
      constructors.push(constructor(p, leading)?);
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

  let mut decl = DataDecl {
    name,
    generics: Vec::new(),
    constructors,
    derives,
    is_newtype,
    is_union,
    visibility,
    span: (start, p.prev_span().1),
    leading,
  };

  // Doc 18 §5.3's generic list, from the field types. After the declaration
  // is assembled because it reads every constructor, and before
  // `newtype_shape` only because that check does not care either way.
  generalise_data(&mut decl);

  // Plan 53 leaf 2. On the finished declaration, because `is_newtype`'s
  // narrowing is a fact about the constructor *list* — the field count alone
  // would miss `newtype Multi = A | B`, which emitted an enum.
  newtype_shape(&decl)?;

  Ok(decl)
}
