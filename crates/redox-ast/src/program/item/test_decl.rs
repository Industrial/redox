use crate::Block;

use redox_lexer::{Span, Trivia};

/// A `test` block — doc 03 §15, kept by doc 18 §8.
///
/// ```text
/// test "adds two numbers"      #[cfg(test)]
///   assert! \-> add 1 2 == 3        mod __rdx_tests {
///                          →       use super::*;
///                                  #[test]
///                                  fn adds_two_numbers() {
///                                      assert_eq!(add(1, 2), 3);
///                                  }
///                              }
/// ```
///
/// Named by a **description**, not an identifier, and the function name is a
/// deterministic slug of it. That is the construct's whole reason: a test's
/// name is prose in every language that reads well, and Rust makes it an
/// identifier.
///
/// Property-test and benchmark blocks are **declined** by §15 for doc 03
/// §2's dependency reason — both lower to calls into crates the user must
/// declare, so either would compile only in projects that happen to have
/// them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestDecl {
  /// The description, as its **raw** source literal, delimiters included.
  ///
  /// Kept raw because it serves two purposes: the slug is derived from it,
  /// and a future `--list` would print it. Decoding here would lose the
  /// author's spelling for the second.
  pub description: String,
  pub body: Block,
  pub span: Span,
  pub leading: Vec<Trivia>,
}
