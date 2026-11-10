use crate::Code;

/// A lint an author can switch off by name — plan 38, plan 53 leaf 4, plan 54
/// leaf 3.
///
/// ```text
/// #[redox::allow(unwrap)]                        -- the attribute, plan 53
/// port = raw.unwrap ()
///
/// n = next ()  -- redox: allow shadow            -- the comment, deprecated
/// ```
///
/// # Why a type rather than a string
///
/// Plan 53 hard-coded `unwrap` into the attribute's spelling, the comment's
/// spelling, the driver's two predicates and the emitter's one — five places,
/// one of which lives in a different crate. Plan 54 §10.6 names that as the
/// reason to land the pragma work *first*: a second lint added against those
/// five strings is a second set of five, and the pair only has to disagree
/// once for an author's suppression to stop suppressing while the attribute
/// still reaches rustc.
///
/// So the name is written down once, here, and every spelling is derived from
/// it. [`all`](Self::all) is what makes recognition total: a lint added to
/// this enum is recognised by the emitter without the emitter being edited,
/// which is the half that was easiest to forget.
///
/// # Why this crate
///
/// Two crates have to agree on the attribute's spelling and neither can see
/// the other. The driver recognises it to **suppress** the warning; the
/// emitter recognises it to **drop** it, because plan 53 §11.6 measured what
/// reaching rustc costs: `error[E0433]: cannot find module or crate 'redox'` —
/// a hard error, not an unknown lint. `redox-diag` is the only crate both
/// depend on directly, and a lint's suppression spelling is part of what its
/// [`Code`] publishes.
///
/// # Exactly one spelling each, after a trim and nothing else
///
/// `#[redox :: allow( unwrap )]` is **not** accepted. A spaced `::` is a
/// diagnostic everywhere else in the language — plan 52's
/// [`Code::SpacedPathSeparator`] — so there is no admitted spelling to be
/// generous towards, and a normaliser would be a second thing the emitter and
/// the driver must agree on rather than a first. An unrecognised spelling
/// fails **loudly**: it is not consumed, so it reaches rustc and `E0433`
/// names it on the author's own line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lint {
  /// `RDX1001` — doc 03 §12.7's `unwrap` discouragement, plan 38.
  Unwrap,
  /// `RDX1003` — a bare `=` rebinding a name already bound in the same
  /// statement list, plan 54 leaf 3.
  Shadow,
}

impl Lint {
  /// Every suppressible lint, which is what makes recognition total.
  pub fn all() -> &'static [Lint] {
    &[Self::Unwrap, Self::Shadow]
  }

  /// The name an author writes inside the pragma.
  ///
  /// Short and not the code, because a pragma is read far more often than it
  /// is written and `#[redox::allow(RDX1003)]` says nothing to a reader who
  /// does not already know the catalogue.
  pub fn name(self) -> &'static str {
    match self {
      Self::Unwrap => "unwrap",
      Self::Shadow => "shadow",
    }
  }

  /// The code this lint raises, for a diagnostic that has to cite it.
  pub fn code(self) -> Code {
    match self {
      Self::Unwrap => Code::UnwrapUsed,
      Self::Shadow => Code::ShadowedBinding,
    }
  }

  /// The suppression attribute as the **lexer stores it**.
  ///
  /// The source spelling is `#[redox::allow(unwrap)]`; an attribute trivium
  /// carries the text *between* the brackets, which is this.
  ///
  /// The `redox::` namespace is load-bearing: `#[allow(unwrap)]` goes on
  /// meaning rustc's lint and nothing else, so Redox's own attributes can
  /// never collide with a Rust one.
  pub fn allow_attribute(self) -> String {
    format!("redox::allow({})", self.name())
  }

  /// The **deprecated** comment spelling, as the lexer stores it.
  ///
  /// The source spelling is `-- redox: allow unwrap`; a comment trivium
  /// carries the text after the dashes. Still a suppression, and
  /// additionally an `RDX1002` — plan 53 §1.3 reverses plan 38 §1.3 on
  /// evidence plan 38 did not have, because plan 48 found `redox fmt`
  /// deleting comments from a `.sdx` and a suppression a formatter can remove
  /// is worse than no suppression.
  ///
  /// The `redox:` namespace is load-bearing here too: `allow shadow` alone
  /// would turn an ordinary English sentence into a pragma by accident.
  pub fn allow_comment(self) -> String {
    format!("redox: allow {}", self.name())
  }

  /// The lint an attribute trivium's text suppresses, if it is one of these.
  ///
  /// The emitter's half of the shared spelling: an attribute that answers
  /// `Some` is Redox's own and is **consumed**, and one that answers `None`
  /// is the author's and is passed through.
  pub fn from_allow_attribute(text: &str) -> Option<Self> {
    Self::all()
      .iter()
      .copied()
      .find(|lint| text == lint.allow_attribute())
  }

  /// The lint a comment trivium's text suppresses, if it is one of these.
  pub fn from_allow_comment(text: &str) -> Option<Self> {
    Self::all()
      .iter()
      .copied()
      .find(|lint| text == lint.allow_comment())
  }
}

#[cfg(test)]
mod tests {
  use super::Lint;

  /// The spellings plan 53 published, which must not move: a program in
  /// outside code carries them as text.
  #[test]
  fn the_published_spellings_are_unchanged() {
    assert_eq!(Lint::Unwrap.allow_attribute(), "redox::allow(unwrap)");
    assert_eq!(Lint::Unwrap.allow_comment(), "redox: allow unwrap");
  }

  /// Recognition is the inverse of the spelling, for every lint — which is
  /// the property hard-coded strings could not have.
  #[test]
  fn every_lint_round_trips_through_both_spellings() {
    for lint in Lint::all() {
      assert_eq!(
        Lint::from_allow_attribute(&lint.allow_attribute()),
        Some(*lint)
      );
      assert_eq!(Lint::from_allow_comment(&lint.allow_comment()), Some(*lint));
    }
  }

  /// Two names would mean one pragma suppressing two lints.
  #[test]
  fn every_name_is_unique() {
    let mut names: Vec<&str> = Lint::all().iter().map(|l| l.name()).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(before, names.len(), "two lints share a pragma name");
  }

  /// A name is one lowercase word, because it is written inside brackets and
  /// compared as exact text.
  #[test]
  fn every_name_is_one_lowercase_word() {
    for lint in Lint::all() {
      let name = lint.name();
      assert!(
        name.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
        "{name} is not a lowercase word"
      );
    }
  }

  /// The cross-spelling confusion the enum exists to prevent: a comment's
  /// text is not an attribute's and never suppresses as one.
  #[test]
  fn the_two_spellings_do_not_cross() {
    assert_eq!(Lint::from_allow_attribute("redox: allow unwrap"), None);
    assert_eq!(Lint::from_allow_comment("redox::allow(unwrap)"), None);
  }
}
