/// Strip the common leading indentation from a `"""` block's body.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 5; doc 01 §7.2 requires
/// removing common indentation while preserving relative indentation.
///
/// Blank lines do not count toward the common prefix — otherwise one empty
/// line would force the prefix to zero and dedent nothing.
pub(crate) fn dedent_block(body: &str) -> String {
  let lines: Vec<&str> = body.lines().collect();

  let common = lines
    .iter()
    .filter(|l| !l.trim().is_empty())
    .map(|l| l.len() - l.trim_start_matches(' ').len())
    .min()
    .unwrap_or(0);

  let mut out = String::new();
  for (i, line) in lines.iter().enumerate() {
    if i > 0 {
      out.push('\n');
    }
    if line.trim().is_empty() {
      // A blank line stays blank rather than keeping stray spaces, which
      // would otherwise show up as trailing whitespace in generated Rust.
      continue;
    }
    out.push_str(&line[common.min(line.len())..]);
  }
  out
}

#[cfg(test)]
mod tests {
  use super::dedent_block;

  #[test]
  fn strips_the_common_prefix_and_keeps_relative_indentation() {
    let body = "    indented\n      relative\n    back";
    assert_eq!(dedent_block(body), "indented\n  relative\nback");
  }

  #[test]
  fn a_blank_line_does_not_force_the_prefix_to_zero() {
    let body = "    one\n\n    two";
    assert_eq!(dedent_block(body), "one\n\ntwo");
  }

  #[test]
  fn leaves_an_unindented_body_alone() {
    assert_eq!(dedent_block("a\nb"), "a\nb");
  }

  #[test]
  fn handles_an_empty_body() {
    assert_eq!(dedent_block(""), "");
  }

  #[test]
  fn strips_trailing_spaces_from_a_whitespace_only_line() {
    assert_eq!(dedent_block("  a\n    \n  b"), "a\n\nb");
  }
}
