use crate::program::item::visibility::Visibility;
use crate::{Expr, Ident, TypeExpr};
use redox_lexer::{Span, Trivia};

/// A module-scope `const` or `static` — doc 03 §13.4, plan 16.
///
/// ```text
/// const MAX :: usize = 1024          const MAX: usize = 1024;
/// static COUNT :: u32 = 0        →   static COUNT: u32 = 0;
/// export const LIMIT :: usize = 8    pub const LIMIT: usize = 8;
/// ```
///
/// Doc 03 §13.4 corrects doc 01 §4.3 on two counts: *"Rust forbids `let` at
/// module scope, so every top-level binding must be `const` or `static`
/// regardless of case; and Rust `const` and `static` require an explicit
/// type, so the specification's `PI = 3.14159` cannot be lowered at all."*
///
/// So the type is written, after the same `::` every other type in Redox uses
/// (doc 18 §5.2), and *"the uppercase-name convention is a lint, not
/// semantics"* — a lowercase one is legal here and rustc's
/// `non_upper_case_globals` is what an author sees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstDecl {
  pub name: Ident,
  pub ty: TypeExpr,
  /// Always present. A class's associated constant may be declared without
  /// one and supplied by an instance; a module-scope binding has nowhere else
  /// for a value to come from, so the parser requires it and says so more
  /// clearly than rustc would.
  pub value: Expr,
  /// `static` rather than `const`.
  ///
  /// One node with a flag, because the two differ by one keyword and nothing
  /// else — and doc 03 §13.4 treats them as one construct with `const` as the
  /// default.
  pub is_static: bool,
  pub visibility: Visibility,
  pub span: Span,
  pub leading: Vec<Trivia>,
}
