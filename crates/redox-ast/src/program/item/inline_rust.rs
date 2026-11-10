use redox_lexer::Span;

use redox_lexer::Trivia;

/// Doc 01 §23.1's inline Rust escape hatch — plan 45 leaf 1.
///
/// ```text
/// rust """
///   unsafe {
///       std::ptr::read(p)
///   }
/// """
/// ```
///
/// The one construct Redox does **not** interpret. `code` is the `"""` block's
/// body exactly as written — undecoded, because escape decoding would turn
/// `println!("a\nb")` into Rust with a real newline inside a string literal —
/// and the emitter writes it out unchanged.
///
/// That makes it the only node that can carry anything, including something
/// that does not compile. Plan 45 §2 argues it still passes doc 03 §2's
/// admission test: it preserves semantics *because* nothing here is analysed,
/// and what rustc says about the contents is what the user gets.
///
/// Before plan 45 this syntax parsed as a **call** to a function named `rust`
/// and emitted `rust("…")`. The hatch doc 03 §18 called the resting place for
/// every unreachable construct was a trap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineRust {
  /// The block's body, verbatim.
  pub code: String,
  pub span: Span,
  pub leading: Vec<Trivia>,
}
