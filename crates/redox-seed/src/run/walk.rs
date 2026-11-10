//! Every directory under the source root, paired with its output directory.

use std::path::Path;
use std::path::PathBuf;

/// `(src_dir, out_dir)` for the root and every directory beneath it.
///
/// # Why the seed walks and `compile_dir` does not
///
/// **`compile_dir` is flat.** Measured at plan 72 leaf 1: pointed at
/// `crates/redox-ast/sdx`, it wrote **7** files where the crate has 53, because
/// `sdx_in` lists one directory and does not descend. Plan 09 built it for a
/// *user's* crate and `examples/cargo-project/src/` holds a single
/// `greeting.sdx`, so nothing had ever handed it a nested module tree —
/// and `tests/port.rs` sidesteps it entirely by invoking `redox build` once
/// per file.
///
/// That is a real gap in the cargo integration and not only in the seed: a
/// user whose `src/` has subdirectories gets a silent partial build from the
/// documented `build.rs` path. It is **not fixed here**, because
/// `compile_dir`'s cache is keyed by file stem — `cache/store.rs` writes
/// `{key}.rs` — so two `expr.sdx` in different directories would collide, and
/// re-keying a cache is a change to shared code that wants its own row.
///
/// Walking here costs nothing and needs no such change: one
/// `compile_dir_with` call per directory keeps every cache key unique, because
/// the store is scoped to the output directory it was handed.
pub(crate) fn walk(src: &Path, out: &Path) -> Vec<(PathBuf, PathBuf)> {
  let mut pairs = vec![(src.to_path_buf(), out.to_path_buf())];
  let mut at = 0;
  while at < pairs.len() {
    let (from, into) = pairs[at].clone();
    at += 1;
    let Ok(entries) = std::fs::read_dir(&from) else {
      continue;
    };
    let mut children: Vec<PathBuf> = entries
      .flatten()
      .map(|e| e.path())
      .filter(|p| p.is_dir())
      .collect();
    // Sorted, so the file list the seed prints is the same on every machine.
    // A bootstrap log that reorders itself is a diff nobody can read.
    children.sort();
    for child in children {
      let Some(name) = child.file_name() else {
        continue;
      };
      // Joined before the push, not inside it: `name` borrows `child`, and
      // moving `child` into the tuple while that borrow is live is `E0505`.
      let target = into.join(name);
      pairs.push((child, target));
    }
  }
  pairs
}

#[cfg(test)]
mod tests {
  use super::walk;

  /// Inline because `walk` is private, per AGENTS.md's rule that inline tests
  /// are for private constructs.
  #[test]
  fn it_mirrors_a_nested_tree() {
    let root = std::path::Path::new("../redox-ast/sdx");
    if !root.is_dir() {
      // The crate layout moved; `port.rs` is what grades that, not this.
      return;
    }
    let pairs = walk(root, std::path::Path::new("/out"));
    assert!(
      pairs.len() > 1,
      "redox-ast's sdx tree is nested, so walking it must find more than the root"
    );
    let (_, out) = &pairs[1];
    assert!(
      out.starts_with("/out"),
      "every output directory mirrors under the root it was given, got {}",
      out.display()
    );
  }
}
