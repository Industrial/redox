//! The per-file line cap.
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`), leaf 1; doc 02 §5.4.
//!
//! Emerald's `crates/emerald-codegen/src/lib.rs` is **34,429 lines in one
//! file** and `emerald-sema/src/lib.rs` is 22,589 — the one clear structural
//! defect in an otherwise well-organised repository. By contrast its
//! `emerald-driver` is 2,658 lines across four named modules and is by far
//! the most readable crate in it.
//!
//! This check costs nothing now and is unenforceable later, which is the
//! whole argument for adding it in the first plan. It lives as an ordinary
//! `#[test]` so it runs under the normal gate rather than needing a separate
//! CI step somebody has to remember to wire up.

use std::path::{Path, PathBuf};

/// Doc 02 §5.4's figure.
const MAX_LINES: usize = 1_500;

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Collect every `.rs` file under `crates/`, which is all the Rust this
/// workspace owns. `target/` is excluded by construction.
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
  let entries = match std::fs::read_dir(dir) {
    Ok(e) => e,
    Err(_) => return,
  };
  for entry in entries {
    let path = entry.expect("should read a directory entry").path();
    if path.is_dir() {
      sources(&path, out);
    } else if path.extension().is_some_and(|e| e == "rs") {
      out.push(path);
    }
  }
}

#[test]
fn no_source_file_exceeds_the_line_cap() {
  let mut files = Vec::new();
  sources(&workspace_root().join("crates"), &mut files);
  files.sort();

  assert!(
    !files.is_empty(),
    "should find the workspace's Rust sources"
  );

  let offenders: Vec<String> = files
    .iter()
    .filter_map(|p| {
      let text = std::fs::read_to_string(p).ok()?;
      let lines = text.lines().count();
      (lines > MAX_LINES).then(|| format!("{} ({lines} lines)", p.display()))
    })
    .collect();

  assert!(
    offenders.is_empty(),
    "these files exceed the {MAX_LINES}-line cap; split them into named modules:\n  {}",
    offenders.join("\n  ")
  );
}
