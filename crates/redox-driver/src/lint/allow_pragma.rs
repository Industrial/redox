use redox_lexer::{Trivia, TriviaKind};

/// Whether one trivium is `RDX1001`'s suppression pragma — plan 38 §1.3.
///
/// ```text
/// port = raw.unwrap ()  -- redox: allow unwrap
/// ```
///
/// A **comment**, not an attribute. Plan 38 §1.3 rejected both attribute
/// forms: passed through, `#[allow(unwrap)]` makes rustc say `unknown lint`,
/// so a program that suppressed a Redox lint would stop compiling; consumed
/// instead, plan 18 §1.3's *attributes are trivia* rule becomes conditional
/// for a single spelling. A comment reaches the `.xrs` as a `//` line and
/// means nothing to rustc, which is exactly what is wanted.
///
/// The `redox:` namespace is load-bearing, and
/// `an_unrelated_comment_does_not_suppress` asserts it: `allow unwrap` alone
/// would turn an ordinary English sentence into a pragma by accident.
///
/// Only a **plain** comment. A doc comment describes the item for a reader of
/// the generated Rust, and giving it a second job would mean a sentence about
/// `unwrap` in prose could silence a warning.
pub(crate) fn allow_pragma(trivium: &Trivia) -> bool {
  matches!(trivium.kind, TriviaKind::LineComment) && trivium.text.trim() == "redox: allow unwrap"
}
