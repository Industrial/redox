/// The message `RDX1003` carries — plan 54 leaf 3.
///
/// It names **the name**, because the whole content of this warning is
/// *which* identifier is bound twice: the line it points at looks exactly
/// like an assignment, and the only way to see that it is not one is to know
/// that the same name appeared above.
///
/// The second clause says what the statement *does* rather than what it
/// failed to do. Plan 54 §1.1 measured this as the one construct in the
/// language caught by nothing — `i = j` compiles, runs, and does something
/// other than what it reads as — so the message has to supply the reading the
/// author did not get from the source.
///
/// Beside [`UNWRAP_MESSAGE`](super::unwrap_message::UNWRAP_MESSAGE), for the
/// same reason it is a named item rather than a literal at the construction
/// site: the wording is the feature. A function rather than a constant
/// because this one interpolates.
pub(crate) fn shadow_message(name: &str) -> String {
  format!(
    "`{name}` is already bound in this block — a bare `=` binds a second name rather than \
     reassigning the first (plan 54 §1.1)"
  )
}

#[cfg(test)]
mod tests {
  use super::shadow_message;

  /// The name reaches the message, which is the reason it is a function.
  #[test]
  fn it_names_the_shadowed_binding() {
    assert!(shadow_message("cursor").starts_with("`cursor` is already bound"));
  }
}
