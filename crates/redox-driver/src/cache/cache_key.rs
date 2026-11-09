use crate::cache::fingerprint::Fingerprint;

/// The cache key for one source file.
///
/// Plan 10 (`docs/15 - 10 - Incremental Cache.md`) leaf 1. `blake3::Hash` is
/// returned bare, which doc 02 §7.1 names as one of the two details worth
/// stealing from Emerald: it is already `Copy`, `Eq`, `Hash` and
/// `Display`-as-hex, so it serves as a map key and a filename with no wrapper
/// type and no boilerplate.
///
/// The key covers the **source bytes and the compiler's fingerprint**, and
/// nothing else. Not the file's path: two files with identical contents should
/// share a result, and a file that moves should not lose one. Not the output
/// directory: the same source compiles to the same Rust wherever it is
/// written. Not the mtime — content, not timestamps, which is what makes this
/// a *content*-addressed cache and immune to a checkout that rewrites
/// modification times.
pub(crate) fn cache_key(source: &str, fingerprint: &Fingerprint) -> blake3::Hash {
  let mut hasher = blake3::Hasher::new();
  // A length prefix keeps the two fields from running together: without it,
  // sources differing only in where the fingerprint appears to begin could
  // collide. Cheap, and the alternative is a subtle one.
  hasher.update(&(source.len() as u64).to_le_bytes());
  hasher.update(source.as_bytes());
  hasher.update(fingerprint.as_bytes());
  hasher.finalize()
}

#[cfg(test)]
mod tests {
  use super::cache_key;
  use crate::cache::fingerprint::Fingerprint;

  fn fp() -> Fingerprint {
    Fingerprint::of(b"compiler")
  }

  #[test]
  fn the_same_source_and_compiler_give_the_same_key() {
    assert_eq!(
      cache_key("main :: ()\nmain =\n", &fp()),
      cache_key("main :: ()\nmain =\n", &fp())
    );
  }

  /// Criterion 2's mechanism: one byte changes the key.
  #[test]
  fn one_changed_byte_changes_the_key() {
    assert_ne!(cache_key("print 1\n", &fp()), cache_key("print 2\n", &fp()));
  }

  /// Criterion 3's mechanism: a new compiler invalidates everything.
  #[test]
  fn a_different_compiler_changes_the_key() {
    let a = cache_key("main :: ()\nmain =\n", &Fingerprint::of(b"old"));
    let b = cache_key("main :: ()\nmain =\n", &Fingerprint::of(b"new"));
    assert_ne!(a, b);
  }

  /// The length prefix exists to prevent this: two inputs whose concatenations
  /// would be identical must still key differently.
  #[test]
  fn the_boundary_between_fields_is_unambiguous() {
    assert_ne!(
      cache_key("ab", &Fingerprint::of(b"c")),
      cache_key("a", &Fingerprint::of(b"bc"))
    );
  }

  /// A key is usable as a filename, which is what the bare `blake3::Hash`
  /// buys and what the store relies on.
  #[test]
  fn renders_as_hex() {
    let hex = cache_key("x", &fp()).to_string();
    assert_eq!(hex.len(), 64);
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
  }
}
