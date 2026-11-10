use crate::ParseError;
use redox_ast::{ConstructorFields, DataDecl};
use redox_diag::Code;
use redox_lexer::Span;

/// Refuse a `newtype` that is not one constructor over one field — `RDX0111`,
/// plan 53 leaf 2.
///
/// Haskell's `newtype` is `data` narrowed to **exactly one constructor with
/// exactly one field**, and that narrowing is the whole of what the keyword
/// says in Redox too: `#[repr(transparent)]` is a promise only a single-field
/// type can keep. Until this check the flag `data_decl` sets was never read
/// again, so `newtype Bad = Bad f64 f64` emitted a two-field tuple struct and
/// `newtype Multi = A | B` emitted an `enum`.
///
/// Checked on the **finished declaration** rather than by bounding
/// `constructor_fields`'s loop, which is where a field limit would otherwise
/// go. The loop serves `data` as well and must stay unbounded for it, and the
/// constructor-count half is not visible from inside a single constructor at
/// all — `redox-fmt`'s `data_text` and `redox-lsp`'s `symbol_of` both rely on
/// the two halves together.
///
/// The shape is read off the declaration, needing no resolver and no types —
/// doc 03 §2's admission test 1, the same basis as doc 18 §2.4's
/// struct-or-enum rule.
pub(crate) fn newtype_shape(decl: &DataDecl) -> Result<(), ParseError> {
  if !decl.is_newtype {
    return Ok(());
  }

  // The constructor count comes first so `newtype Multi = A | B` is reported
  // as the enum it would have become, rather than as `A` carrying no field.
  // The blamed span is the **second** constructor: the first is the one a
  // `newtype` is allowed, so underlining the lot would accuse it too.
  if decl.constructors.len() > 1 {
    return Err(ParseError::with(
      Code::NewtypeShape,
      format!(
        "a `newtype` has exactly one constructor, and this declaration has {} \
         — write `data` for a choice between constructors",
        decl.constructors.len()
      ),
      decl.constructors[1].span,
    ));
  }

  // `newtype Lone`, with no `=` row at all, parses and emitted `struct Lone;`
  // — the same wrong shape as a constructor carrying no field, so the two
  // share a message and differ only in what the caret can point at.
  let (blame, fields): (Span, Vec<Span>) = match decl.constructors.first() {
    None => (decl.name.span, Vec::new()),
    Some(only) => (
      only.span,
      match &only.fields {
        ConstructorFields::Unit => Vec::new(),
        ConstructorFields::Tuple(types) => types.iter().map(|t| t.span).collect(),
        // A one-field record `newtype Tag = Tag { raw : String }` is legal
        // Haskell and a legal `repr(transparent)` struct, so records are
        // counted rather than refused.
        ConstructorFields::Record(decls) => decls.iter().map(|f| f.span).collect(),
      },
    ),
  };

  match fields.len() {
    1 => Ok(()),
    // rustc is no help here: `#[repr(transparent)] pub struct Unit;` compiles
    // clean (plan 53 §11.3), so this branch is the only thing that catches a
    // payload-free `newtype`.
    0 => Err(ParseError::with(
      Code::NewtypeShape,
      "a `newtype` wraps exactly one field, and this declaration wraps none \
       — write `data` for a type with no payload",
      blame,
    )),
    n => Err(ParseError::with(
      Code::NewtypeShape,
      format!(
        "a `newtype` wraps exactly one field, and this declaration wraps {n} \
         — write `data` for a constructor with more than one field"
      ),
      fields[1],
    )),
  }
}
