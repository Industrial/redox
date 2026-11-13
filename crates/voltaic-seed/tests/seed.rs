//! The seed reproduces a ported crate's committed Rust.
//!
//! Plan 72 leaf 1. This is the assertion that makes the seed worth checking
//! in: given a crate's `.sdx` tree, the seed must produce exactly the `.rs`
//! that is committed under that crate's `src/` — because on a clean clone with
//! that `src/` untracked, what the seed produces is all there is.
//!
//! # Why this is not `tests/port.rs`
//!
//! `port.rs` grades the **committed** Rust against the `.sdx`, by invoking
//! `voltaic build` once per file and then rustfmt. It proves the two trees agree.
//! This proves the **seed** agrees with them — a different claim, because the
//! seed is a different binary with a different dependency graph, a directory
//! walk of its own, and no `clap`, `notify` or terminal.
//!
//! The two can disagree. `walk`'s doc records the first way they did:
//! `compile_dir` does not descend, so the seed's first run wrote 7 of 53 files
//! while `port.rs` was green.
//!
//! # Why rustfmt runs here
//!
//! The committed Rust is formatted with **this repository's** config, and
//! Voltaic targets rustfmt's defaults. Plan 72 §6 declines to move that into the
//! emitter — a canonical emitter would need a configuration model for a
//! formatter it does not own — so the pipeline is seed, then rustfmt, and
//! `init.sh` will do the same on a clean clone.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// The workspace root, from this crate's manifest.
fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `.rs` under `dir`, relative to it, sorted.
fn rust_files(dir: &Path) -> Vec<PathBuf> {
  let mut found = Vec::new();
  collect(dir, dir, &mut found);
  found.sort();
  found
}

fn collect(root: &Path, dir: &Path, into: &mut Vec<PathBuf>) {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if path.is_dir() {
      collect(root, &path, into);
    } else if path.extension().is_some_and(|e| e == "rs") {
      into.push(
        path
          .strip_prefix(root)
          .expect("under the root being walked")
          .to_path_buf(),
      );
    }
  }
}

/// `voltaic-ast`, because it is the smaller of the two ported crates and its
/// `.sdx` tree is nested three deep — which is the shape that found
/// `compile_dir`'s flat walk.
#[test]
fn the_seed_reproduces_a_ported_crates_committed_rust() {
  let root = workspace_root();
  let sdx = root.join("crates/voltaic-ast/sdx");
  let committed = root.join("crates/voltaic-ast/src");

  let out = root.join("target/voltaic-seed-test");
  let _ = std::fs::remove_dir_all(&out);
  std::fs::create_dir_all(&out).expect("should create the output directory");

  let run = Command::new(env!("CARGO_BIN_EXE_voltaic-seed"))
    .arg(&sdx)
    .arg(&out)
    .output()
    .expect("should run voltaic-seed");
  assert!(
    run.status.success(),
    "the seed should transpile voltaic-ast's source:\n{}",
    String::from_utf8_lossy(&run.stderr)
  );

  // The cache the driver writes into its output directory, which `tidy`
  // removes. Asserted rather than assumed: `one_construct_per_file` walks for
  // `*.rs` with no hidden-directory exclusion, so a cache left under a crate's
  // `src/` would fail that test with a message about constructs rather than
  // about caching.
  assert!(
    !out.join(".voltaic-cache").exists(),
    "the seed must not leave its incremental cache in the output tree"
  );

  let produced = rust_files(&out);
  let expected = rust_files(&committed);
  assert_eq!(
    produced, expected,
    "the seed must produce exactly the files that are committed — \
     `compile_dir` does not descend, so a flat walk silently produces a subset"
  );

  for rel in &produced {
    let path = out.join(rel);
    let formatted = Command::new("rustfmt")
      .args(["--edition", "2024", "--config", "skip_children=true"])
      .arg(&path)
      .status()
      .expect("should run rustfmt");
    assert!(
      formatted.success(),
      "rustfmt should format {}",
      rel.display()
    );

    let got = std::fs::read_to_string(&path).expect("should read the seed's output");
    let want =
      std::fs::read_to_string(committed.join(rel)).expect("should read the committed Rust");
    assert_eq!(
      got,
      want,
      "{} differs from what is committed. On a clean clone the seed's output is \
       all there is, so a difference here is a crate that would not compile.",
      rel.display()
    );
  }
}
