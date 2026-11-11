//! Enforces `docs/06 - Redox Code Architecture.md` §2: directory structure.
//!
//! Plan 91 retired one-construct-per-file and with it four of this file's
//! seven checks — the module-head rule and the depth cap describe a tree that
//! no longer exists. The three that remain never depended on it:
//!
//! * `no_directory_groups_by_item_kind` is the hazard of merging 572 files
//!   into 52. The lazy merge is a `helpers/` directory, and this refuses it.
//! * `no_module_uses_mod_rs` and `no_directory_is_empty` are layout hygiene,
//!   orthogonal to how many constructs a file holds.
//!
//! Deleting the file wholesale would have discarded three working checks in
//! order to retire one rule.

use std::path::{Path, PathBuf};

/// Directory names that group by item kind or act as buckets (doc 06 §2.2).
const BANNED_DIRS: [&str; 13] = [
  "types", "structs", "enums", "traits", "impls", "helpers", "utils", "util", "common", "misc",
  "shared", "internal", "private",
];

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every crate's `src/` directory.
fn crate_sources() -> Vec<PathBuf> {
  let crates = workspace_root().join("crates");
  let mut out: Vec<PathBuf> = std::fs::read_dir(&crates)
    .expect("crates/ should exist")
    .map(|e| e.expect("should read a directory entry").path().join("src"))
    .filter(|p| p.is_dir())
    .collect();
  out.sort();
  out
}

/// Whether `path` is a hidden entry — a tooling artifact such as `.hermes`,
/// never a source module.
fn is_hidden(path: &Path) -> bool {
  path
    .file_name()
    .is_some_and(|n| n.to_string_lossy().starts_with('.'))
}

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  for entry in entries {
    let path = entry.expect("should read a directory entry").path();
    if is_hidden(&path) {
      continue;
    }
    if path.is_dir() {
      rs_files(&path, out);
    } else if path.extension().is_some_and(|e| e == "rs") {
      out.push(path);
    }
  }
}

fn dirs(dir: &Path, out: &mut Vec<PathBuf>) {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  for entry in entries {
    let path = entry.expect("should read a directory entry").path();
    if path.is_dir() && !is_hidden(&path) {
      out.push(path.clone());
      dirs(&path, out);
    }
  }
}

/// Doc 06 §2.2: directories name domain concepts, never item kinds.
#[test]
fn no_directory_groups_by_item_kind() {
  let mut offenders = Vec::new();
  for src in crate_sources() {
    let mut found = Vec::new();
    dirs(&src, &mut found);
    for dir in found {
      let name = dir
        .file_name()
        .expect("a directory name")
        .to_string_lossy()
        .to_string();
      if BANNED_DIRS.contains(&name.as_str()) {
        offenders.push(format!(
          "{} groups by item kind or is a bucket; name the domain concept instead",
          dir.display()
        ));
      }
    }
  }
  assert!(
    offenders.is_empty(),
    "doc 06 §2.2 bans kind-based and bucket directory names:\n  {}",
    offenders.join("\n  ")
  );
}

/// Doc 06 §1.3: the 2018-and-later layout, never `mod.rs`.
#[test]
fn no_module_uses_mod_rs() {
  let mut offenders = Vec::new();
  for src in crate_sources() {
    let mut files = Vec::new();
    rs_files(&src, &mut files);
    for path in files {
      if path.file_name().is_some_and(|n| n == "mod.rs") {
        offenders.push(path.display().to_string());
      }
    }
  }
  assert!(
    offenders.is_empty(),
    "doc 06 §1.3 bans mod.rs; use foo.rs beside foo/:\n  {}",
    offenders.join("\n  ")
  );
}

/// Doc 06 §2.5: a directory with no `.rs` files in it at all is a leftover.
#[test]
fn no_directory_is_empty() {
  let mut offenders = Vec::new();
  for src in crate_sources() {
    let mut found = Vec::new();
    dirs(&src, &mut found);
    for dir in found {
      let mut files = Vec::new();
      rs_files(&dir, &mut files);
      if files.is_empty() {
        offenders.push(dir.display().to_string());
      }
    }
  }
  assert!(
    offenders.is_empty(),
    "doc 06 §2.5: a directory with no children is a deleted directory:\n  {}",
    offenders.join("\n  ")
  );
}
