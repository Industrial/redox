/// Strip the single optional space after a comment's `#`.
///
/// `# text` and `#text` both yield `text`, so the emitter can write `// text`
/// uniformly without re-inspecting the original spacing.
pub(crate) fn trim_comment(rest: &str) -> String {
  rest.strip_prefix(' ').unwrap_or(rest).to_string()
}

#[cfg(test)]
mod tests {
  use super::trim_comment;

  #[test]
  fn strips_one_leading_space_and_no_more() {
    assert_eq!(trim_comment(" text"), "text");
    assert_eq!(trim_comment("text"), "text");
    assert_eq!(trim_comment("  text"), " text");
    assert_eq!(trim_comment(""), "");
  }
}
