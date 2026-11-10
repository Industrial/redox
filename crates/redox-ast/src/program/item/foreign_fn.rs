use crate::{Ident, Signature};

use redox_lexer::{Span, Trivia};

/// One declaration inside an `extern` block — plan 45 leaf 3.
///
/// A name and a signature and nothing else, which is the same shape doc 18
/// §2.5's required method has: `abs : i32 -> i32` becomes
/// `fn abs(__rdx_a0: i32) -> i32;`. It reuses
/// [`signature_only`](../../../../redox_parser) for that reason — a foreign
/// declaration has no equation to take parameter names from, so the
/// generated names doc 02 reserves are what the parameters get.
///
/// Distinct from [`FnDecl`](crate::FnDecl) because a foreign function has no
/// body at all, where a `FnDecl` with no clauses would be a defect everywhere
/// else in the compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignFn {
  pub name: Ident,
  pub sig: Signature,
  pub span: Span,
  pub leading: Vec<Trivia>,
}
