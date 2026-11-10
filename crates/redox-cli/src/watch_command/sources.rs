use std::path::Path;

/// Every `.sdx` directly in `dir`, sorted.
///
/// Plan 12 (`docs/17 - 12 - Watcher.md`) leaf 3. The same rule
/// `redox_driver`'s `sdx_in` applies — one level deep, sorted — so
/// `redox watch <dir>` compiles the same set of files a build script would,
/// and a user does not discover the two disagree.
///
/// Sorted for the same reason it is sorted there: unsorted directory order
/// would make the diagnostics reorder between runs for no reason.
pub(crate) fn sources(dir: &Path) -> Vec<String> {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return Vec::new();
  };
  let mut found: Vec<String> = entries
    .filter_map(Result::ok)
    .map(|e| e.path())
    .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "sdx"))
    .map(|p| p.display().to_string())
    .collect();
  found.sort();
  found
}

#[cfg(test)]
mod tests {
  use super::sources;

  #[test]
  fn finds_and_sorts_sdx_files() {
    let dir = std::env::temp_dir().join(format!("redox-watch-src-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("should create");
    for name in ["b.sdx", "a.sdx", "ignored.rs"] {
      std::fs::write(dir.join(name), "main : ()\nmain =\n  print 1\n").expect("should write");
    }

    let found = sources(&dir);
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(
      found[0].ends_with("a.sdx") && found[1].ends_with("b.sdx"),
      "{found:?}"
    );

    std::fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn an_unreadable_directory_yields_nothing() {
    assert!(sources(std::path::Path::new("/nonexistent/redox")).is_empty());
  }
}
