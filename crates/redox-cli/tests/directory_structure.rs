//! Enforces `docs/06 - Redox Code Architecture.md` §2: directory structure.
//!
//! §1 alone produces a correct but shapeless crate — forty files in one
//! directory with no indication of what owns what. These checks make the
//! module tree part of the design rather than an accident.

use std::path::{Path, PathBuf};

/// Directory names that group by item kind or act as buckets (doc 06 §2.2).
const BANNED_DIRS: [&str; 13] = [
  "types", "structs", "enums", "traits", "impls", "helpers", "utils", "util", "common", "misc",
  "shared", "internal", "private",
];

/// Doc 06 §2.4.
const MAX_DEPTH: usize = 3;

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

/// Doc 06 §2.1: a file directly under `src/` is legal only as the module root
/// or as a module head — a construct with a sibling directory of the same name.
#[test]
fn every_root_level_file_is_a_module_head() {
  let mut offenders = Vec::new();
  for src in crate_sources() {
    let Ok(entries) = std::fs::read_dir(&src) else {
      continue;
    };
    for entry in entries {
      let path = entry.expect("should read a directory entry").path();
      if path.is_dir() || path.extension().is_none_or(|e| e != "rs") {
        continue;
      }
      let stem = path
        .file_stem()
        .expect("a file stem")
        .to_string_lossy()
        .to_string();
      if stem == "lib" || stem == "main" {
        continue;
      }
      if !src.join(&stem).is_dir() {
        offenders.push(format!(
          "{} is not a module head: create {}/ for its children, or move it under an \
           existing module",
          path.display(),
          src.join(&stem).display()
        ));
      }
    }
  }
  assert!(
    offenders.is_empty(),
    "doc 06 §2.1 requires every root-level file to be a module head:\n  {}",
    offenders.join("\n  ")
  );
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

/// Doc 06 §2.4: depth mirrors containment, and a fourth level means the crate
/// boundary should have split instead.
#[test]
fn no_file_is_deeper_than_the_depth_cap() {
  let mut offenders = Vec::new();
  for src in crate_sources() {
    let mut files = Vec::new();
    rs_files(&src, &mut files);
    for path in files {
      let depth = path
        .strip_prefix(&src)
        .expect("file is under src/")
        .components()
        .count();
      if depth > MAX_DEPTH {
        offenders.push(format!("{} is {depth} levels below src/", path.display()));
      }
    }
  }
  assert!(
    offenders.is_empty(),
    "doc 06 §2.4 caps depth at {MAX_DEPTH} levels below src/:\n  {}",
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
