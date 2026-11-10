use std::path::{Path, PathBuf};

/// Every `.sdx` directly in `dir`, sorted.
///
/// Plan 09 (`docs/14 - 09 - Cargo Integration.md`) leaf 1.
///
/// **Sorted**, because a build script's output must not depend on directory
/// iteration order: an unsorted list would compile files in a different order
/// on different machines, and the diagnostics a user sees would reorder
/// between runs for no reason.
///
/// **One level deep.** Nesting would need module support to mean anything —
/// a `.sdx` in a subdirectory has no way to declare itself part of the crate
/// — so recursing would create files nothing can reach. Plan 09 §6 records
/// this.
///
/// A missing or unreadable directory yields nothing rather than failing. A
/// crate with no `.sdx` at all is a legitimate state during setup, and a build
/// script that dies on it would make adding Redox to an existing crate a
/// two-step operation that fails in between.
pub(crate) fn sdx_in(dir: &Path) -> Vec<PathBuf> {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return Vec::new();
  };
  let mut found: Vec<PathBuf> = entries
    .filter_map(Result::ok)
    .map(|e| e.path())
    .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "sdx"))
    .collect();
  found.sort();
  found
}

#[cfg(test)]
mod tests {
  use super::sdx_in;

  /// A scratch directory holding the named files, removed on drop.
  struct Dir(std::path::PathBuf);

  impl Dir {
    fn with(tag: &str, names: &[&str]) -> Self {
      use std::sync::atomic::{AtomicUsize, Ordering};
      static NEXT: AtomicUsize = AtomicUsize::new(0);
      let n = NEXT.fetch_add(1, Ordering::Relaxed);
      let dir = std::env::temp_dir().join(format!("redox-sdx-in-{tag}-{}-{n}", std::process::id()));
      std::fs::create_dir_all(&dir).expect("should create");
      for name in names {
        std::fs::write(dir.join(name), "main : ()\nmain =\n  print 1\n").expect("should write");
      }
      Self(dir)
    }
  }

  impl Drop for Dir {
    fn drop(&mut self) {
      std::fs::remove_dir_all(&self.0).ok();
    }
  }

  #[test]
  fn finds_sdx_files_and_sorts_them() {
    let dir = Dir::with("sorted", &["b.sdx", "a.sdx", "c.sdx"]);
    let found: Vec<String> = sdx_in(&dir.0)
      .iter()
      .map(|p| p.file_name().expect("a name").to_string_lossy().to_string())
      .collect();
    assert_eq!(found, vec!["a.sdx", "b.sdx", "c.sdx"]);
  }

  #[test]
  fn ignores_other_extensions() {
    let dir = Dir::with("mixed", &["keep.sdx", "skip.rs", "skip.xrs", "skip.md"]);
    assert_eq!(sdx_in(&dir.0).len(), 1);
  }

  #[test]
  fn yields_nothing_for_a_missing_directory() {
    assert!(sdx_in(std::path::Path::new("/nonexistent/redox")).is_empty());
  }

  #[test]
  fn yields_nothing_for_an_empty_directory() {
    let dir = Dir::with("empty", &[]);
    assert!(sdx_in(&dir.0).is_empty());
  }
}
