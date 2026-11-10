use redox_diag::Lint;
use redox_lexer::{Trivia, TriviaKind};

/// The lint one trivium suppresses as a **deprecated comment** — plan 38
/// §1.3, deprecated by plan 53 leaf 5, parametrised by plan 54 leaf 3.
///
/// ```text
/// port = raw.unwrap ()  -- redox: allow unwrap
/// n = next ()           -- redox: allow shadow
/// ```
///
/// Still a suppression, and additionally an `RDX1002`. Plan 38 chose a
/// comment because it *"reaches nothing downstream"*; plan 53 §1.3 reverses
/// that on evidence plan 38 did not have — plan 48 found `redox fmt`
/// **deleting** comments from a `.sdx`, so a comment is unsafe *upstream*
/// even though it is harmless downstream, and a suppression a formatter can
/// remove is worse than no suppression. The attribute spelling is
/// [`attribute_pragma`](super::attribute_pragma::attribute_pragma).
///
/// The `redox:` namespace is load-bearing, and
/// `an_unrelated_comment_does_not_suppress` asserts it: `allow unwrap` alone
/// would turn an ordinary English sentence into a pragma by accident.
///
/// Only a **plain** comment. A doc comment describes the item for a reader of
/// the generated Rust, and giving it a second job would mean a sentence about
/// `unwrap` in prose could silence a warning.
pub(crate) fn comment_pragma(trivium: &Trivia) -> Option<Lint> {
  if !matches!(trivium.kind, TriviaKind::LineComment) {
    return None;
  }
  Lint::from_allow_comment(trivium.text.trim())
}

#[cfg(test)]
mod tests {
  use super::comment_pragma;
  use redox_diag::Lint;
  use redox_lexer::{Trivia, TriviaKind};

  fn trivium(kind: TriviaKind, text: &str) -> Trivia {
    Trivia {
      kind,
      text: text.to_string(),
      span: (0, 0),
    }
  }

  #[test]
  fn the_pragma_suppresses() {
    assert_eq!(
      comment_pragma(&trivium(TriviaKind::LineComment, "redox: allow unwrap")),
      Some(Lint::Unwrap)
    );
  }

  /// The second lint reaches the deprecated spelling too, because an author
  /// who learned the comment form has no reason to know it is per-lint.
  #[test]
  fn each_lint_is_suppressed_by_its_own_name() {
    assert_eq!(
      comment_pragma(&trivium(TriviaKind::LineComment, "redox: allow shadow")),
      Some(Lint::Shadow)
    );
  }

  /// The `redox:` namespace, which is why the pragma is not a sentence.
  #[test]
  fn an_unrelated_comment_does_not_suppress() {
    assert_eq!(
      comment_pragma(&trivium(
        TriviaKind::LineComment,
        "allow unwrap here, it cannot fail"
      )),
      None
    );
  }

  /// A doc comment describes the item for a reader of the generated Rust;
  /// giving it a second job would let prose about `unwrap` silence a warning.
  #[test]
  fn a_doc_comment_does_not_suppress() {
    assert_eq!(
      comment_pragma(&trivium(TriviaKind::DocComment, "redox: allow unwrap")),
      None
    );
  }
}
