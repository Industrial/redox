use crate::TypeExpr;
use crate::program::item::field_decl::FieldDecl;

/// What a constructor carries — doc 18 §2.4.
///
/// Three shapes, and they are Rust's three: a unit, a tuple, and a record.
/// Haskell's declaration form maps onto Rust's "with almost nothing left
/// over", which is §2.4's claim and is why this enum has no fourth variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstructorFields {
  /// `Close` → `Close` / `struct Marker;`
  Unit,
  /// `Key char` → `Key(char)` / `struct Meters(f64);`
  ///
  /// The field types are juxtaposed, like an application — which is the same
  /// reading doc 18 §2.1 gives everywhere else, so a constructor needs no
  /// notation of its own.
  Tuple(Vec<TypeExpr>),
  /// `Click { x : i32, y : i32 }` → `Click { x: i32, y: i32 }`
  ///
  /// Braces for the shape and `::` for the types, which is the same `::` as
  /// a signature line and an ascription (doc 18 §5.2). Records use `=` for
  /// *values*, which is what makes the pairing legible: `::` introduces a
  /// type, `=` introduces a value, in a declaration and a literal alike.
  Record(Vec<FieldDecl>),
}
