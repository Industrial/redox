use redox_ast::{Constraint, TypeExpr};

/// Everything a signature line says before its equations.
///
/// Plan 63 (`docs/57 - 63 - Instance Inference.md`) leaf 0 and leaf 2. The
/// clause loop needs all five of these to build a clause's signature, and an
/// `instance` member that **omits** its signature needs to borrow them from
/// the class — so they travel as one value rather than as five arguments
/// threaded through two call sites.
///
/// # Why the spine and not the rendered `Signature`
///
/// A rendered `Signature` cannot be re-used, and §11.2 is why: `receiver` has
/// already dropped the marked parameter's type by then, so the receiver's
/// spelling is gone and the spine is unrecoverable from the stored member. The
/// head is captured **before** `receiver` runs, which is the only moment the
/// information still exists.
#[derive(Debug, Clone)]
pub(crate) struct SignatureHead {
  /// The spine's types, as `parse_spine` split them — doc 18 §2.2's top-level
  /// arrows, already ignoring the ones nested inside a verbatim type.
  pub types: Vec<TypeExpr>,
  /// Doc 18 §2.2's `=>` clause.
  pub bounds: Vec<Constraint>,
  /// Whether the signature was marked `const`.
  pub is_const: bool,
  /// Whether the signature was marked `async`.
  pub is_async: bool,
  /// Whether the signature was marked `unsafe`.
  pub is_unsafe: bool,
}
