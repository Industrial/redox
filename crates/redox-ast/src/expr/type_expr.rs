use redox_lexer::Span;

/// A type, captured as its **verbatim source text**.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) §1.1. Doc 03 §14.1 fixes type
/// expressions as copied through unchanged, so every present and future Rust
/// type form is reachable for free and nothing can drift. A partial type
/// grammar would be more code and strictly less capable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeExpr {
  /// The type's source text, exactly as written.
  pub text: String,
  pub span: Span,
}
