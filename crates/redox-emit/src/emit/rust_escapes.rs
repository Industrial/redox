/// Re-encode a Redox string literal's text as a Rust string literal's text.
///
/// Plan 08 (`docs/13 - 08 - Run Expectations.md`) §10.1 — a defect the run
/// corpus caught on its first execution.
///
/// Redox's string escapes are Rust's, with **one** addition: `\#`, which
/// escapes the `#` that would otherwise open an interpolation (doc 01 §7.1).
/// Rust has no `\#`, and rejects it:
///
/// ```text
/// error: unknown character escape: `#`
/// ```
///
/// The parser carries a literal's *raw* text through to the emitter — correct
/// for every escape the two languages share, and wrong for exactly this one.
/// So this function translates `\#` to a bare `#`, which is what Rust spells
/// the same character as, and leaves every other backslash sequence alone.
///
/// Why not decode the whole literal and re-encode it from
/// `TokenKind::Str`'s `parts`? Because that would mean Redox re-encoding
/// every escape Rust already understands, and each one is a chance to get a
/// character wrong in a way no test would notice. Translating the single
/// escape that differs is a smaller claim and a verifiable one.
pub(crate) fn rust_escapes(raw: &str) -> String {
  let mut out = String::with_capacity(raw.len());
  let mut chars = raw.chars();
  while let Some(ch) = chars.next() {
    if ch != '\\' {
      out.push(ch);
      continue;
    }
    match chars.next() {
      // The one escape Rust does not have. `\#` means `#`, and `#` needs no
      // escaping in Rust, so the backslash is dropped.
      Some('#') => out.push('#'),
      // Every other escape is Rust's too; pass both characters through. This
      // includes `\\`, which must consume its second backslash here or a
      // following `#` would be read as an escape it is not part of.
      Some(other) => {
        out.push('\\');
        out.push(other);
      }
      // A trailing backslash cannot occur in a token the lexer accepted, but
      // emitting it unchanged is better than dropping a character.
      None => out.push('\\'),
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::rust_escapes;

  #[test]
  fn translates_the_one_escape_rust_lacks() {
    assert_eq!(rust_escapes(r#""a\#{b""#), r#""a#{b""#);
  }

  #[test]
  fn leaves_every_shared_escape_alone() {
    for escape in [
      r"\n",
      r"\t",
      r"\r",
      r"\0",
      r#"\""#,
      r"\'",
      r"\x41",
      r"\u{1F600}",
    ] {
      assert_eq!(rust_escapes(escape), escape, "{escape} should be untouched");
    }
  }

  /// The case a naive `replace("\\#", "#")` gets wrong: a literal backslash
  /// followed by a `#` is not a `\#` escape, and must keep its backslash.
  #[test]
  fn does_not_mistake_an_escaped_backslash_for_an_escaped_hash() {
    assert_eq!(rust_escapes(r"a\\#b"), r"a\\#b");
    assert_eq!(rust_escapes(r"a\\\#b"), r"a\\#b");
  }

  #[test]
  fn leaves_text_with_no_escapes_alone() {
    assert_eq!(rust_escapes("hello world"), "hello world");
    assert_eq!(rust_escapes(""), "");
  }

  #[test]
  fn survives_a_trailing_backslash() {
    assert_eq!(rust_escapes(r"a\"), r"a\");
  }
}
