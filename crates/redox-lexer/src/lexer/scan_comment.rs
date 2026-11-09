use crate::trivia::kind::TriviaKind;

/// Which comment form a `--` run opens, and how many characters introduce it.
///
/// Doc 18 §5.1 — the Haskell-shaped comment surface. `--` replaces `#`, which
/// retires four of doc 03 §4.1's six meanings of `#` and reduces that
/// longest-match table to a line.
///
/// | Written | Kind | Rust |
/// |---|---|---|
/// | `-- line` | line comment | `// line` |
/// | `-- \| outer` | outer doc | `/// outer` |
/// | `-- ! inner` | inner doc | `//! inner` |
///
/// `-- |` is Haddock's own outer-doc marker. `-- !` is doc 18's invention,
/// since Haskell has no module-level analogue of `//!`.
///
/// **`--` opens a comment only when followed by whitespace, a line ending, or
/// one of the two doc markers.** Doc 18 §5.1 names the hazard this avoids —
/// `x - -1` against `x --1` — and gives this as the mitigation. The artifact
/// doc 18 was written from states the hazard and the mitigation in successive
/// sentences and they disagree about `--1`; the mitigation is the operative
/// rule, and the stricter reading is taken deliberately so that `x --1` is
/// arithmetic rather than a silently truncated line. Recorded rather than
/// resolved silently, because real Haskell takes the looser reading.
pub(crate) fn scan_comment(rest: &str) -> Option<(TriviaKind, usize)> {
  let after = rest.strip_prefix("--")?;

  // `-- |` and `-- !` are the doc forms. Checked before the generic rule
  // because the marker is what makes them comments at all.
  if let Some(body) = after.strip_prefix(" |") {
    let _ = body;
    return Some((TriviaKind::DocComment, 4));
  }
  if let Some(body) = after.strip_prefix(" !") {
    let _ = body;
    return Some((TriviaKind::InnerDocComment, 4));
  }

  // A plain line comment needs whitespace or the end of the line after the
  // dashes. A third dash is still a comment — `---` is a divider people write
  // — so an all-dash run counts as whitespace-terminated.
  let opens = match after.chars().next() {
    None => true,
    Some(c) => c.is_whitespace() || c == '-',
  };
  opens.then_some((TriviaKind::LineComment, 2))
}

#[cfg(test)]
mod tests {
  use super::scan_comment;
  use crate::trivia::kind::TriviaKind;

  #[test]
  fn a_dash_dash_space_opens_a_line_comment() {
    assert_eq!(scan_comment("-- hello"), Some((TriviaKind::LineComment, 2)));
  }

  #[test]
  fn a_bare_dash_dash_at_end_of_line_opens_one() {
    assert_eq!(scan_comment("--"), Some((TriviaKind::LineComment, 2)));
    assert_eq!(scan_comment("--\n"), Some((TriviaKind::LineComment, 2)));
  }

  /// A divider is a comment, not an operator.
  #[test]
  fn a_run_of_dashes_opens_one() {
    assert_eq!(scan_comment("-----"), Some((TriviaKind::LineComment, 2)));
  }

  #[test]
  fn the_doc_markers_are_recognised() {
    assert_eq!(
      scan_comment("-- | outer"),
      Some((TriviaKind::DocComment, 4))
    );
    assert_eq!(
      scan_comment("-- ! inner"),
      Some((TriviaKind::InnerDocComment, 4))
    );
  }

  /// The hazard doc 18 §5.1 names, taken in the strict reading: this is
  /// arithmetic, not a comment.
  #[test]
  fn dashes_against_a_digit_are_not_a_comment() {
    assert_eq!(scan_comment("--1"), None);
  }

  #[test]
  fn dashes_against_an_identifier_are_not_a_comment() {
    assert_eq!(scan_comment("--x"), None);
  }

  #[test]
  fn a_single_dash_is_not_a_comment() {
    assert_eq!(scan_comment("-"), None);
    assert_eq!(scan_comment("- 1"), None);
    assert_eq!(scan_comment("-> x"), None);
  }

  #[test]
  fn text_that_is_not_dashes_is_not_a_comment() {
    assert_eq!(scan_comment("hello"), None);
    assert_eq!(scan_comment(""), None);
  }
}
