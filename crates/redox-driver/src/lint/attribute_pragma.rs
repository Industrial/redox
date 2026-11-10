use redox_diag::ALLOW_UNWRAP_ATTRIBUTE;
use redox_lexer::{Trivia, TriviaKind};

/// Whether one trivium is `RDX1001`'s suppression attribute — plan 53 leaf 4.
///
/// ```text
/// #[redox::allow(unwrap)]
/// port = raw.unwrap ()
/// ```
///
/// The spelling lives in `redox-diag` as [`ALLOW_UNWRAP_ATTRIBUTE`], because
/// the emitter has to recognise the same string to **drop** it: an attribute
/// that reaches rustc is `error[E0433]: cannot find module or crate 'redox'`,
/// so a driver that suppresses a spelling the emitter does not consume
/// produces a program that no longer compiles. That constant's own
/// documentation carries the argument, including why whitespace inside the
/// brackets is not normalised.
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
pub(crate) fn attribute_pragma(trivium: &Trivia) -> bool {
  matches!(trivium.kind, TriviaKind::Attribute) && trivium.text.trim() == ALLOW_UNWRAP_ATTRIBUTE
}

#[cfg(test)]
mod tests {
  use super::attribute_pragma;
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
    assert!(attribute_pragma(&trivium(
      TriviaKind::Attribute,
      "redox::allow(unwrap)"
    )));
  }

  /// The namespace, which is what keeps `#[allow(unwrap)]` rustc's.
  #[test]
  fn an_unnamespaced_allow_is_rustc_s() {
    assert!(!attribute_pragma(&trivium(
      TriviaKind::Attribute,
      "allow(unwrap)"
    )));
  }

  /// The whitespace decision, pinned rather than left to a reader: one
  /// spelling, and a generous one reaches rustc as `E0433` rather than being
  /// silently ignored.
  #[test]
  fn whitespace_inside_the_brackets_is_not_normalised() {
    assert!(!attribute_pragma(&trivium(
      TriviaKind::Attribute,
      "redox :: allow( unwrap )"
    )));
  }

  /// An *inner* attribute applies to the enclosing module, not to the next
  /// construct, so it is not this pragma.
  #[test]
  fn an_inner_attribute_does_not_suppress() {
    assert!(!attribute_pragma(&trivium(
      TriviaKind::InnerAttribute,
      "redox::allow(unwrap)"
    )));
  }
}
