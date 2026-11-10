use crate::Ident;
use crate::program::item::visibility::Visibility;
use redox_lexer::{Span, Trivia};

/// A `module` declaration — doc 18 §7's replacement for `mod`, plan 14 §1.2.
///
/// ```text
/// module config              mod config;
/// export module parser       pub mod parser;
///
/// module tests               mod tests {
///   test "it parses"     →       #[cfg(test)]
///     assert! \-> 1 == 1                mod __rdx_tests { … }
///                            }
/// ```
///
/// Doc 18 §8 declines Haskell's module *header* — `module M (f, g) where`
/// "cannot express Rust's per-item, per-field, scoped visibility" — which
/// leaves one reading, and §9's own Rust column confirms it: **`module` is
/// Rust's `mod`.** A bodiless declaration names a sibling file's module; one
/// with a body is doc 03 §13.5's inline block, *"needed for the
/// near-universal test-module convention"*.
///
/// Both are passthrough. What is not passthrough is that a body makes the
/// item emitter a tree rather than a loop — plan 14 §6 discloses it, and the
/// risk it carries is plan 06's source map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDecl {
  pub name: Ident,
  /// The body's items, or **`None`** for a bodiless declaration.
  ///
  /// `None` rather than an empty vector, because `mod name;` and
  /// `mod name {}` are different Rust: the first loads a file and the second
  /// declares an empty module. An empty `Vec` could not tell them apart.
  pub items: Option<Vec<crate::Item>>,
  pub visibility: Visibility,
  pub span: Span,
  pub leading: Vec<Trivia>,
}
