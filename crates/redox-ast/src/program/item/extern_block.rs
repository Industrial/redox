use crate::program::item::foreign_fn::ForeignFn;
use redox_lexer::{Span, Trivia};

/// An `extern` block — plan 45 leaf 3.
///
/// ```text
/// extern "C"
///   abs :: i32 -> i32
///   labs :: i64 -> i64
/// ```
///
/// Doc 01 §26 lists **FFI** among the things that *"must remain possible"*,
/// and doc 03 §23 item 2 states that the inline Rust escape hatch *"is not an
/// acceptable resting place"* for it. So this is a construct, and it is the
/// one leaf of plan 45 the specification names as a requirement rather than
/// as a minor gap.
///
/// `abi` keeps its quotes, as an error variant's message and a test's
/// description do: it is written through to Rust verbatim, and an ABI Rust
/// does not know is **rustc's** to reject, not Redox's — which is the
/// every-check-rustc-makes corollary and the reason no ABI list appears
/// anywhere in this compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternBlock {
  /// The ABI string, quotes included — `"C"`.
  pub abi: String,
  pub decls: Vec<ForeignFn>,
  pub span: Span,
  pub leading: Vec<Trivia>,
}
