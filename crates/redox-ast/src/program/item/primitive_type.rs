/// Whether a lowercase name is one of Rust's **primitive** type names.
///
/// Doc 18 §5.3 says "lowercase type names are type variables", and taken
/// literally that makes `i32` a type variable and `add : i32 -> i32 -> i32`
/// a generic function of one parameter. It cannot mean that, and this closed
/// list is the correction: a lowercase name is a type variable *unless* it is
/// a primitive.
///
/// Closed, lexical and decidable from the source text alone, which is doc 03
/// §2's admission test 1 — and the reason this can be a list rather than a
/// lookup. The cost is that a user cannot name a type in lowercase, which
/// §5.3 was already spending to make type variables undeclared.
///
/// It lives in the tree crate rather than the parser's because **two** places
/// need the same list and a second copy would drift: `type_vars` keeps a
/// primitive out of a function's generic list, and [`TypeApp::is_var`] keeps
/// `instance Convert f64 Meters` from emitting `Convert<F64>`.
///
/// A `case` ladder where the Rust it replaces wrote `matches!`. Plan 64 made
/// macros callable as `name! \-> e`, and `matches!`'s second argument is a
/// **pattern** rather than an expression — so it is one of the macros §6 of
/// that plan discloses as out of reach for a value-passing shape. The `or`
/// ladder is the same closed list and needs no macro at all.
///
/// [`TypeApp::is_var`]: crate::TypeApp::is_var
pub fn primitive_type(name: &str) -> bool {
  match name {
    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => true,
    "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => true,
    "f32" | "f64" | "bool" | "char" | "str" => true,
    _ => false,
  }
}
