/// Escape a string literal's braces for use as a `format!` format string.
///
/// Plan 08 (`docs/13 - 08 - Run Expectations.md`) §10.1 — the second defect
/// the run corpus caught.
///
/// `print "a"` lowers to `println!("a")`, which passes the literal to
/// `println!` as its **format string**. That is fine until the literal
/// contains a brace: `print "see {this}"` would lower to
/// `println!("see {this}")`, where `{this}` is a format placeholder naming a
/// variable that does not exist. rustc then reports an error about a line the
/// user did not write, pointing at an identifier they did not use.
///
/// Doubling each brace is how a format string spells a literal one, so
/// `{` becomes `{{` and `}` becomes `}}`.
///
/// This is deliberately **not** part of [`super::rust_escapes::rust_escapes`],
/// and the distinction is load-bearing: `rust_escapes` makes a literal valid
/// *Rust*, and applies wherever a literal is emitted. Doubling braces makes a
/// literal mean itself as a *format string*, and applies only where one is
/// used as that. A `StrLit` emitted in an ordinary expression position must
/// keep its braces single, or `let s = "{}"` would become `let s = "{{}}"`.
///
/// When interpolation arrives (plan 23), an interpolated part becomes a real
/// `{}` hole *after* this runs on the literal segments, so the two compose:
/// holes the author wrote stay holes, braces they wrote stay braces.
pub(crate) fn double_braces(literal: &str) -> String {
  literal.replace('{', "{{").replace('}', "}}")
}

#[cfg(test)]
mod tests {
  use super::double_braces;

  #[test]
  fn doubles_a_brace_so_it_prints_as_itself() {
    assert_eq!(double_braces(r#""see {this}""#), r#""see {{this}}""#);
  }

  #[test]
  fn leaves_a_literal_with_no_braces_alone() {
    assert_eq!(
      double_braces(r#""Hello, world!""#),
      r#""Hello, world!""#,
      "plan 01's golden depends on this being unchanged"
    );
  }

  /// A brace that is already doubled in the source text is user data too, and
  /// doubling it again is what makes it print as two braces.
  #[test]
  fn doubles_each_brace_independently() {
    assert_eq!(double_braces("{{"), "{{{{");
    assert_eq!(double_braces("}"), "}}");
  }

  #[test]
  fn handles_an_empty_literal() {
    assert_eq!(double_braces(""), "");
  }
}
