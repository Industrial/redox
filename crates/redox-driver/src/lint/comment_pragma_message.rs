use redox_diag::Lint;

/// The message `RDX1002` carries — plan 53 leaf 5, parametrised by plan 54
/// leaf 3.
///
/// It names **the spelling to write**, because that is the whole of what this
/// diagnostic is for: the comment pragma still suppresses, so an author who
/// is told only that it is deprecated has been given no work they can do.
///
/// Both spellings come from [`Lint`] rather than from a literal, so the
/// message cannot name an attribute the emitter does not consume — and the
/// text for `unwrap` is byte-identical to the constant this replaced, which
/// is what keeps plan 53's blessed expectation unchanged.
///
/// Beside the message, [`UNWRAP_MESSAGE`](super::unwrap_message::UNWRAP_MESSAGE)
/// for the same reason — the wording is the feature, so it lives in a named
/// item rather than inline at the construction site.
pub(crate) fn comment_pragma_message(lint: Lint) -> String {
  format!(
    "`-- {}` is deprecated — write `#[{}]` on the line above instead (plan 53 §1.3)",
    lint.allow_comment(),
    lint.allow_attribute()
  )
}

#[cfg(test)]
mod tests {
  use super::comment_pragma_message;
  use redox_diag::Lint;

  /// Plan 53's wording, unchanged: the blessed `.stderr` compares as exact
  /// text, so parametrising must not move a character of it.
  #[test]
  fn the_unwrap_wording_is_plan_53_s() {
    assert_eq!(
      comment_pragma_message(Lint::Unwrap),
      "`-- redox: allow unwrap` is deprecated — write `#[redox::allow(unwrap)]` on the line \
       above instead (plan 53 §1.3)"
    );
  }

  /// The deprecation names the lint it is about, which is why the message
  /// could not stay a constant.
  #[test]
  fn each_lint_gets_its_own_spellings() {
    let text = comment_pragma_message(Lint::Shadow);
    assert!(text.contains("-- redox: allow shadow"));
    assert!(text.contains("#[redox::allow(shadow)]"));
  }
}
