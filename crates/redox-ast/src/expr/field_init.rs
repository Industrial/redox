use crate::{Expr, Ident};

use redox_lexer::Span;

/// One `name = value` field of a record literal — doc 18 §2.4.
///
/// ```text
/// p  = Point { x = 10, y = 20 }    let p = Point { x: 10, y: 20 };
/// q  = Point { x, y }              let q = Point { x: x, y: y };
/// ```
///
/// `=` for a value, against `::` for a type, which is the pairing §2.4 asks
/// for: one token introduces a type wherever it appears, the other introduces
/// a value, in a declaration and a literal alike.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldInit {
  pub name: Ident,
  /// The value, or `None` when the field is **punned**.
  ///
  /// `Point { x, y }` means `Point { x: x, y: y }`. Doc 18 §2.4 notes punning
  /// "came from Haskell in the first place", which is why it survives the
  /// conversion rather than arriving with it. Recorded as an absence rather
  /// than as a synthesised `Expr::Ident` so the formatter can write back what
  /// the author wrote.
  pub value: Option<Expr>,
  pub span: Span,
}
