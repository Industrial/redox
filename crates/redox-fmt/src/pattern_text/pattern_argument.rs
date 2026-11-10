pub(crate) mod stands_alone;

use crate::pattern_text::pattern_argument::stands_alone::stands_alone;

use crate::pattern_text::pattern_text;

use redox_ast::Pattern;

/// One pattern in a position where the next token could extend it —
/// parenthesised unless it cannot be.
///
/// The three such positions: an argument of a constructor pattern, the inner
/// pattern of an `@` binding, and an alternative of an `or`. All three are
/// followed by something that a greedy pattern would swallow.
///
/// # What needs parentheses, and why each one does
///
/// - **`Apply` with arguments.** Juxtaposition is left-associative, so
///   `Some Some v` is `Some` applied to two arguments.
/// - **`Or`.** `Some 1 or 2` would read its `or` at the outer level.
/// - **`As`.** `Some x @ y` binds the `@` to the wrong pattern.
/// - **`Range`.** `Some 0..13` has no reading a reader would agree on.
/// - **`Record`.** `Some Point { x = 0 }` reads the brace as a new
///   construct, which is the same reason a record literal is not a bare
///   expression argument. `examples/draw_condition.sdx` writes
///   `Some (Point { x = 0, y = 0, .. })`.
///
/// An `Apply` with **no** arguments — `None`, `Close` — is a bare name and
/// needs nothing, which is why the arity is checked rather than the variant.
pub(crate) fn pattern_argument(pattern: &Pattern) -> String {
  if stands_alone(pattern) {
    pattern_text(pattern)
  } else {
    format!("({})", pattern_text(pattern))
  }
}
