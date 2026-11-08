/// A hash of the compiler that produced a cached result.
///
/// Plan 10 (`docs/15 - 10 - Incremental Cache.md`) leaf 2, and the single most
/// important three lines in the cache. Doc 02 §7.1 states the reason, lifted
/// from Emerald:
///
/// > so that rebuilding the compiler invalidates every previously-cached
/// > result instead of silently reusing a stale `.o` a different compiler
/// > produced.
///
/// For Redox the stale artifact is worse than a `.o`: it is *generated Rust
/// from an older emitter*, which compiles cleanly, runs, and is wrong. Nothing
/// downstream can detect it, because there is nothing wrong with it except
/// that it is not what the current compiler would produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fingerprint(blake3::Hash);

impl Fingerprint {
  /// The fingerprint of the running compiler.
  ///
  /// If `current_exe` or the read fails, the fingerprint is derived from
  /// something that differs every process, so every lookup misses. That is
  /// the safe direction: a cache that cannot identify its compiler must not
  /// claim its results are reusable.
  pub fn current() -> Self {
    match std::env::current_exe().and_then(std::fs::read) {
      Ok(bytes) => Self::of(&bytes),
      Err(_) => Self::of(&std::process::id().to_le_bytes()),
    }
  }

  /// A fingerprint of arbitrary bytes.
  ///
  /// Public so a test can vary the compiler's identity without rebuilding the
  /// compiler — plan 10 criterion 3 would otherwise be untestable in practice,
  /// and an untested invalidation rule is the kind that quietly stops working.
  pub fn of(bytes: &[u8]) -> Self {
    Self(blake3::hash(bytes))
  }

  pub(crate) fn as_bytes(&self) -> &[u8; 32] {
    self.0.as_bytes()
  }
}

#[cfg(test)]
mod tests {
  use super::Fingerprint;

  #[test]
  fn differs_for_different_bytes() {
    assert_ne!(
      Fingerprint::of(b"compiler-a"),
      Fingerprint::of(b"compiler-b")
    );
  }

  #[test]
  fn is_stable_for_the_same_bytes() {
    assert_eq!(Fingerprint::of(b"same"), Fingerprint::of(b"same"));
  }

  /// The running compiler has *a* fingerprint, and asking twice agrees.
  #[test]
  fn the_current_compiler_has_a_stable_fingerprint() {
    assert_eq!(Fingerprint::current(), Fingerprint::current());
  }
}
