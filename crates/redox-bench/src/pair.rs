pub(crate) mod pairs;

use std::path::PathBuf;

/// One lowering, as the `.sdx` that produces it and the `.rs` a competent Rust
/// author writes for the same job.
///
/// Plan 44 §1.3 narrows the corpus to the lowerings that generate **more** than
/// the author wrote. For a syntactic lowering — `x |> f` becoming `x.f()` — the
/// generated Rust *is* what a human writes, so comparing them measures rustc
/// rather than Redox.
#[derive(Debug, Clone)]
pub struct Pair {
  /// The pair's filename stem, which names the pair and nothing else.
  pub stem: String,
  /// The function both halves define, **discovered** from the hand-written
  /// half rather than derived from the stem. `pairs`' doc records why, and
  /// that a mismatch is silent in the dangerous direction.
  pub function: String,
  pub sdx: PathBuf,
  pub rust: PathBuf,
}
