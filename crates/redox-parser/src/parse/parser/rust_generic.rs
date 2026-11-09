/// A generic parameter's Rust spelling — doc 18 §5.3.
///
/// A lifetime keeps its name; a type variable is uppercased. `'a` stays `'a`
/// and `t` becomes `T`, which is the whole of the transformation and the
/// reason `type_vars` returns the source's names rather than Rust's: a caller
/// needs both, to write the generic list and to rewrite the types.
pub(crate) fn rust_generic(name: &str) -> String {
  if name.starts_with('\'') {
    return name.to_string();
  }
  name.to_uppercase()
}
