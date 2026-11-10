use redox_diag::Lint;
use redox_lexer::{Trivia, TriviaKind};

/// The lint one trivium suppresses as an **attribute** — plan 53 leaf 4,
/// parametrised by plan 54 leaf 3.
///
/// ```text
/// #[redox::allow(unwrap)]        #[redox::allow(shadow)]
/// port = raw.unwrap ()           n = next ()
/// ```
///
/// The spellings live in `redox-diag` as [`Lint::allow_attribute`], because
/// the emitter has to recognise the same strings to **drop** them: an
/// attribute that reaches rustc is
/// `error[E0433]: cannot find module or crate 'redox'`, so a driver that
/// suppresses a spelling the emitter does not consume produces a program that
/// no longer compiles. [`Lint`]'s own documentation carries the argument,
/// including why whitespace inside the brackets is not normalised.
///
/// The `redox::` namespace is load-bearing for the reason the comment form's
/// `redox:` is, and for a second: `#[allow(unwrap)]` goes on meaning rustc's
/// lint and nothing else, so Redox's own attributes can never collide with a
/// Rust one.
///
/// An attribute can only sit on the line **above** what it applies to, which
/// is why this spelling needed the suppression scope widened from a line to a
/// construct's span — see
/// [`suppressed_spans`](super::suppressed_spans::suppressed_spans).
pub(crate) fn attribute_pragma(trivium: &Trivia) -> Option<Lint> {
  if !matches!(trivium.kind, TriviaKind::Attribute) {
    return None;
  }
  Lint::from_allow_attribute(trivium.text.trim())
}

#[cfg(test)]
mod tests {
  use super::attribute_pragma;
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
  fn the_attribute_suppresses() {
    assert_eq!(
      attribute_pragma(&trivium(TriviaKind::Attribute, "redox::allow(unwrap)")),
      Some(Lint::Unwrap)
    );
  }

  /// The second lint, which is the whole of what parametrising bought: one
  /// predicate, and the name decides which warning is silenced.
  #[test]
  fn each_lint_is_suppressed_by_its_own_name() {
    assert_eq!(
      attribute_pragma(&trivium(TriviaKind::Attribute, "redox::allow(shadow)")),
      Some(Lint::Shadow)
    );
  }

  /// A name no lint has is **not** a pragma, so the emitter does not consume
  /// it either and rustc names it — which is the loud failure the
  /// one-spelling rule relies on.
  #[test]
  fn an_unknown_lint_name_is_not_a_pragma() {
    assert_eq!(
      attribute_pragma(&trivium(TriviaKind::Attribute, "redox::allow(unwarp)")),
      None
    );
  }

  /// The namespace, which is what keeps `#[allow(unwrap)]` rustc's.
  #[test]
  fn an_unnamespaced_allow_is_rustc_s() {
    assert_eq!(
      attribute_pragma(&trivium(TriviaKind::Attribute, "allow(unwrap)")),
      None
    );
  }

  /// The whitespace decision, pinned rather than left to a reader: one
  /// spelling, and a generous one reaches rustc as `E0433` rather than being
  /// silently ignored.
  #[test]
  fn whitespace_inside_the_brackets_is_not_normalised() {
    assert_eq!(
      attribute_pragma(&trivium(TriviaKind::Attribute, "redox :: allow( unwrap )")),
      None
    );
  }

  /// An *inner* attribute applies to the enclosing module, not to the next
  /// construct, so it is not this pragma.
  #[test]
  fn an_inner_attribute_does_not_suppress() {
    assert_eq!(
      attribute_pragma(&trivium(TriviaKind::InnerAttribute, "redox::allow(unwrap)")),
      None
    );
  }
}
