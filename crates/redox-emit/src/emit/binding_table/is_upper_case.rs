/// Whether `name` is spelled the way Rust spells a constant.
///
/// Plan 59 §11.8. Sharing a lower-case name earns `non_upper_case_globals`,
/// and the examples corpus requires rustc to emit nothing — so the case
/// decides the item, as it already decides a type variable from a concrete
/// type (doc 18 §5.3) and a constructor from a field
/// (`crate::emit::names_a_constructor`).
///
/// A name with no letters at all is not upper case. Rust has no such
/// identifier, so this is a guard against the empty answer rather than a
/// reachable case.
pub(crate) fn is_upper_case(name: &str) -> bool {
  name.chars().any(char::is_alphabetic) && !name.chars().any(|c| c.is_ascii_lowercase())
}
