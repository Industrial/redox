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
/// [`TypeApp::is_var`]: crate::TypeApp::is_var
pub fn primitive_type(name: &str) -> bool {
  matches!(
    name,
    "i8"
      | "i16"
      | "i32"
      | "i64"
      | "i128"
      | "isize"
      | "u8"
      | "u16"
      | "u32"
      | "u64"
      | "u128"
      | "usize"
      | "f32"
      | "f64"
      | "bool"
      | "char"
      | "str"
  )
}
