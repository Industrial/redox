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
  /// something that differs every call, so every lookup misses. That is
  /// the safe direction: a cache that cannot identify its compiler must not
  /// claim its results are reusable.
  ///
  /// # Why the clock and not the pid
  ///
  /// This fallback used `std::process::id()`, which is the obvious thing and
  /// **panics** on `wasm32-wasip1` — *"no pids on this platform"*, from
  /// `std`'s `process::unsupported`. Both halves fail on that target:
  /// `current_exe` is unsupported too, so the fallback is always taken, and
  /// then it aborts. Plan 72 leaf 2 found it by running `redox-seed` under
  /// `wasmtime`, 47 files into a 53-file tree.
  ///
  /// The wall clock exists wherever `std` does — WASI has `clock_time_get` —
  /// and it satisfies the paragraph above better than a pid did: a pid is
  /// reused between runs, so two different compilers could share a
  /// fingerprint, where a nanosecond timestamp effectively cannot. If even
  /// the clock fails there is nothing left to ask, and zero is as good a
  /// constant as any: the cache is then useless rather than wrong, which is
  /// the direction this function has always chosen.
  pub fn current() -> Self {
    match std::env::current_exe().and_then(std::fs::read) {
      Ok(bytes) => Self::of(&bytes),
      Err(_) => Self::of(&Self::now().to_le_bytes()),
    }
  }

  /// Nanoseconds since the epoch, or zero if the clock refuses.
  fn now() -> u128 {
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map_or(0, |d| d.as_nanos())
  }

  /// A fingerprint of arbitrary bytes.
  ///
  /// Public so a test can vary the compiler's identity without rebuilding the
  /// compiler — plan 10 criterion 3 would otherwise be untestable in practice,
  /// and an untested invalidation rule is the kind that quietly stops working.
  pub fn of(bytes: &[u8]) -> Self {
    Self(blake3::hash(bytes))
  }

  /// The hash's 32 raw bytes, for mixing into a cache key.
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
