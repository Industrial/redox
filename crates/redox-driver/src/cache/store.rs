/// What a cache hit returns: the generated Rust and its map, as text.
pub(crate) mod cached;

use crate::cache::store::cached::Cached;
use std::path::{Path, PathBuf};

/// The on-disk cache under `<out>/.redox-cache/`.
///
/// Plan 10 (`docs/15 - 10 - Incremental Cache.md`) leaf 3.
///
/// **Every failure is a miss.** No method returns an error, and that is the
/// design rather than laziness: by plan 10 §1's governing rule, lifted from
/// Emerald, *a cache hit is strictly an optimization, never a correctness
/// dependency*. An unreadable entry, an unwritable directory, a half-written
/// file from an interrupted build — each one means "compile it properly", and
/// a cache that could instead fail a build would have negative value.
///
/// Entries are two files per key, `<hash>.rs` and `<hash>.rs.map`, read back
/// together. A key with only one of the two present is treated as absent,
/// which is what makes an interrupted write harmless.
pub(crate) struct Store {
  /// The cache directory this store reads and writes under.
  dir: PathBuf,
}

impl Store {
  /// The cache for an output directory.
  ///
  /// Under `OUT_DIR` for a build script, so the cache is per-crate,
  /// per-profile, and emptied by `cargo clean` — it inherits Cargo's lifetime
  /// rules rather than inventing its own, and so needs no eviction policy.
  pub(crate) fn new(out: &Path) -> Self {
    Self {
      dir: out.join(".redox-cache"),
    }
  }

  /// The `<hash>.rs` and `<hash>.rs.map` paths one key names.
  fn paths(&self, key: &blake3::Hash) -> (PathBuf, PathBuf) {
    let rust = self.dir.join(format!("{key}.rs"));
    let map = self.dir.join(format!("{key}.rs.map"));
    (rust, map)
  }

  /// The cached result for `key`, if both halves are present and readable.
  pub(crate) fn get(&self, key: &blake3::Hash) -> Option<Cached> {
    let (rust_path, map_path) = self.paths(key);
    let rust = std::fs::read_to_string(rust_path).ok()?;
    let map = std::fs::read_to_string(map_path).ok()?;
    // A map that does not parse means the entry was written by something other
    // than this format, or was truncated. Either way: miss.
    redox_diag::parse_map(&map).ok()?;
    Some(Cached { rust, map })
  }

  /// Record a result. A failure to write is silently a future miss.
  pub(crate) fn put(&self, key: &blake3::Hash, cached: &Cached) {
    if std::fs::create_dir_all(&self.dir).is_err() {
      return;
    }
    let (rust_path, map_path) = self.paths(key);
    // The map is written *first*. If the process dies between the two writes,
    // the next `get` finds a map with no Rust beside it and treats the entry
    // as absent — whereas the other order could leave Rust with no map, and a
    // caller that read only the Rust would use an entry with no positions.
    if std::fs::write(&map_path, &cached.map).is_err() {
      return;
    }
    if std::fs::write(&rust_path, &cached.rust).is_err() {
      // Remove the orphan so the directory does not accumulate half-entries.
      std::fs::remove_file(&map_path).ok();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::Store;
  use crate::cache::store::cached::Cached;

  struct Dir(std::path::PathBuf);

  impl Dir {
    fn new(tag: &str) -> Self {
      use std::sync::atomic::{AtomicUsize, Ordering};
      static NEXT: AtomicUsize = AtomicUsize::new(0);
      let n = NEXT.fetch_add(1, Ordering::Relaxed);
      let dir = std::env::temp_dir().join(format!("redox-store-{tag}-{}-{n}", std::process::id()));
      std::fs::create_dir_all(&dir).expect("should create");
      Self(dir)
    }
  }

  impl Drop for Dir {
    fn drop(&mut self) {
      std::fs::remove_dir_all(&self.0).ok();
    }
  }

  fn entry() -> Cached {
    Cached {
      rust: "fn main() {}\n".to_string(),
      map: "redox-map 1 a.sdx\n1 1 1 1\n".to_string(),
    }
  }

  #[test]
  fn a_stored_entry_comes_back() {
    let dir = Dir::new("roundtrip");
    let store = Store::new(&dir.0);
    let key = blake3::hash(b"k");
    assert_eq!(store.get(&key), None, "nothing is cached yet");
    store.put(&key, &entry());
    assert_eq!(store.get(&key), Some(entry()));
  }

  #[test]
  fn an_unknown_key_is_a_miss() {
    let dir = Dir::new("unknown");
    let store = Store::new(&dir.0);
    store.put(&blake3::hash(b"a"), &entry());
    assert_eq!(store.get(&blake3::hash(b"b")), None);
  }

  /// Criterion 5: a corrupt entry is a miss, not a failure.
  #[test]
  fn a_corrupt_map_is_a_miss() {
    let dir = Dir::new("corrupt");
    let store = Store::new(&dir.0);
    let key = blake3::hash(b"k");
    store.put(&key, &entry());

    let (_, map_path) = store.paths(&key);
    std::fs::write(&map_path, "not a redox map at all\n").expect("should overwrite");
    assert_eq!(store.get(&key), None);
  }

  /// A half-entry — the Rust missing — is absent rather than partly usable.
  #[test]
  fn a_half_written_entry_is_a_miss() {
    let dir = Dir::new("half");
    let store = Store::new(&dir.0);
    let key = blake3::hash(b"k");
    store.put(&key, &entry());

    let (rust_path, _) = store.paths(&key);
    std::fs::remove_file(&rust_path).expect("should remove");
    assert_eq!(store.get(&key), None);
  }

  /// An unwritable cache directory must not fail anything.
  #[test]
  fn an_unwritable_store_is_simply_always_a_miss() {
    let store = Store::new(std::path::Path::new("/proc/nonexistent-redox"));
    let key = blake3::hash(b"k");
    store.put(&key, &entry());
    assert_eq!(store.get(&key), None);
  }
}
