/// Split an interpolation body into its expression and its format spec.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 4. Doc 03 §11.1 carries
/// Rust's own format-spec grammar inside the interpolation, so `#{amount:.2}`
/// becomes `format!("{:.2}", amount)`.
///
/// The split is at the **first depth-zero `:` that is not part of `::`**. Both
/// guards are load-bearing and both have a worked example in the tests:
///
/// - the depth test keeps `#{Point{x: 1}}`'s struct literal intact;
/// - the `::` test keeps `#{HashMap::new()}`'s path intact.
///
/// Taking the *last* colon instead would break `#{a:>8}` the moment a spec
/// contained one.
pub(crate) fn split_format_spec(body: &str) -> (&str, Option<&str>) {
  let bytes = body.as_bytes();
  let mut depth = 0i32;
  let mut i = 0usize;

  while i < bytes.len() {
    match bytes[i] {
      b'(' | b'[' | b'{' => depth += 1,
      b')' | b']' | b'}' => depth -= 1,
      b':' if depth == 0 => {
        // Skip `::` entirely — both colons, so the second is not re-examined.
        if bytes.get(i + 1) == Some(&b':') {
          i += 2;
          continue;
        }
        // A `:` cannot be a spec separator if it closes a `::` we are mid-way
        // through, which `i += 2` above already prevents.
        return (&body[..i], Some(&body[i + 1..]));
      }
      _ => {}
    }
    i += 1;
  }
  (body, None)
}

#[cfg(test)]
mod tests {
  use super::split_format_spec;

  #[test]
  fn finds_a_plain_spec() {
    assert_eq!(split_format_spec("amount:.2"), ("amount", Some(".2")));
    assert_eq!(split_format_spec("name:>8"), ("name", Some(">8")));
    assert_eq!(split_format_spec("cfg:?"), ("cfg", Some("?")));
  }

  #[test]
  fn returns_no_spec_when_there_is_none() {
    assert_eq!(split_format_spec("name"), ("name", None));
    assert_eq!(split_format_spec("a + b"), ("a + b", None));
  }

  /// The `::` guard.
  #[test]
  fn keeps_a_path_intact() {
    assert_eq!(
      split_format_spec("HashMap::new()"),
      ("HashMap::new()", None)
    );
    assert_eq!(
      split_format_spec("String::from(x):>4"),
      ("String::from(x)", Some(">4"))
    );
  }

  /// The depth guard.
  #[test]
  fn keeps_a_nested_colon_intact() {
    assert_eq!(split_format_spec("Point{x: 1}"), ("Point{x: 1}", None));
    assert_eq!(split_format_spec("map[k]"), ("map[k]", None));
    assert_eq!(
      split_format_spec("Point{x: 1}:?"),
      ("Point{x: 1}", Some("?"))
    );
  }

  /// The first depth-zero colon wins, not the last.
  #[test]
  fn takes_the_first_eligible_colon() {
    assert_eq!(split_format_spec("a:>8:x"), ("a", Some(">8:x")));
  }
}
