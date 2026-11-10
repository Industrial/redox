//! A clone with no generated Rust can be bootstrapped from the committed seed.
//!
//! Plan 72 leaf 5, and the precondition that row set itself: *"never before CI
//! has rebuilt that crate from the seed on a clean clone."* There is no CI
//! configuration in this repository — the gate is the pre-push hook — so the
//! clean-clone proof is a test, like `status.rs` and `plan_orientation.rs`.
//!
//! # What it does
//!
//! Clones `HEAD` into `target/`, **deletes** the generated trees in the clone,
//! runs `./bootstrap.sh` there, and requires every file to come back
//! byte-identical to what this working tree has.
//!
//! Deleting rather than relying on them being untracked is deliberate. It makes
//! the test independent of commit order — it proves the claim that matters,
//! *a tree without the generated Rust can be rebuilt from the seed*, whether or
//! not the ignore flip is in `HEAD` yet — and it is strictly the harder case,
//! since a clone that still carried them would pass trivially.
//!
//! `assert_the_generated_trees_are_ignored` covers the other half: that they
//! really are untracked in this working tree.
//!
//! # What it does not do
//!
//! Run `cargo check` in the clone. A fresh dependency graph is minutes of
//! build for an assertion the ordinary gate already makes about the same bytes
//! — this test establishes that the bytes are the same, and `:check` and
//! `:test` establish that those bytes compile and pass. Splitting it that way
//! keeps a clean-clone proof in the gate instead of out of it.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// The crates whose `src/` is generated. Mirrors `bootstrap.sh`'s `PORTED`.
const PORTED: &[&str] = &["redox-ast", "redox-fmt"];

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

#[test]
fn a_clone_without_the_generated_rust_bootstraps_from_the_seed() {
  let root = workspace_root()
    .canonicalize()
    .expect("the workspace root should exist");
  let clone = root.join("target/redox-bootstrap-clone");
  let _ = std::fs::remove_dir_all(&clone);

  let cloned = Command::new("git")
    .args(["clone", "--quiet", "--no-hardlinks", "--depth", "1"])
    .arg(&root)
    .arg(&clone)
    .output()
    .expect("should run git clone");
  assert!(
    cloned.status.success(),
    "should clone the workspace:\n{}",
    String::from_utf8_lossy(&cloned.stderr)
  );

  // The state a clean clone is in once the ignore has flipped, reached by
  // deletion so this does not depend on which commit is `HEAD`.
  for crate_name in PORTED {
    let generated = clone.join("crates").join(crate_name).join("src");
    std::fs::remove_dir_all(&generated)
      .unwrap_or_else(|e| panic!("should clear {}: {e}", generated.display()));
    assert!(
      !generated.exists(),
      "{} should be gone before bootstrapping",
      generated.display()
    );
  }

  let boot = Command::new("./bootstrap.sh")
    .current_dir(&clone)
    .output()
    .expect("should run ./bootstrap.sh in the clone");
  assert!(
    boot.status.success(),
    "./bootstrap.sh should rebuild the generated crates from the seed:\n{}\n{}",
    String::from_utf8_lossy(&boot.stdout),
    String::from_utf8_lossy(&boot.stderr)
  );

  for crate_name in PORTED {
    let here = root.join("crates").join(crate_name).join("src");
    let there = clone.join("crates").join(crate_name).join("src");

    let want = rust_files(&here);
    let got = rust_files(&there);
    assert_eq!(
      got, want,
      "{crate_name}: bootstrapping a clone produced a different set of files. \
       `compile_dir` does not descend on its own, so a flat walk silently \
       produces a subset (plan 72 §11.2)."
    );

    for rel in &want {
      let expected = std::fs::read_to_string(here.join(rel)).expect("should read this tree's Rust");
      let actual = std::fs::read_to_string(there.join(rel)).expect("should read the clone's Rust");
      assert_eq!(
        actual,
        expected,
        "{crate_name}/{} differs after bootstrapping. On a clean clone this file is \
         all there is, so a difference here is a crate that would not compile.",
        rel.display()
      );
    }
  }
}

/// The generated trees really are untracked here.
///
/// The other half of leaf 5: the test above proves a tree *without* them can be
/// rebuilt, and this proves this tree is that kind of tree.
///
/// # Both halves, because the first one alone lied
///
/// This asked only `git check-ignore` at first, on the reasoning that the
/// question is whether the rule exists. It is not. `check-ignore` answers
/// *"does a rule match this path"* and says nothing about the index — and git
/// deliberately ignores `.gitignore` for a path that is **already tracked**,
/// which is the entire reason `git rm --cached` exists.
///
/// So when leaf 5's flip half-failed — the rules committed, the 110 files
/// still in the index, because a `prek` hook's stash-and-restore cycle reset
/// the staged deletions — this test passed. It asserted the state it could see
/// most easily rather than the state it cared about.
///
/// Both now have to hold. A rule with tracked files is a flip that did not
/// finish; untracked files with no rule would come back on the next `git add`.
#[test]
fn the_generated_trees_are_ignored() {
  let root = workspace_root();
  for crate_name in PORTED {
    let dir = format!("crates/{crate_name}/src");
    let path = format!("{dir}/lib.rs");

    let checked = Command::new("git")
      .args(["check-ignore", "-q", &path])
      .current_dir(&root)
      .status()
      .expect("should run git check-ignore");
    assert!(
      checked.success(),
      "{path} should be gitignored: its crate is written in Redox and the `.rs` is \
       generated. If this fails the ignore flip has been reverted, and the repository is \
       committing ~32,000 lines of output with no author (plan 72 §1.3)."
    );

    let tracked = Command::new("git")
      .args(["ls-files", "--", &dir])
      .current_dir(&root)
      .output()
      .expect("should run git ls-files");
    let listed = String::from_utf8_lossy(&tracked.stdout);
    let count = listed.lines().filter(|l| !l.trim().is_empty()).count();
    assert_eq!(
      count,
      0,
      "{dir} still has {count} file(s) in the index. A `.gitignore` rule does not \
       untrack anything — git ignores the rule for a path it already tracks — so the \
       flip needs `git rm -r --cached {dir}` and a commit that actually carries the \
       deletions. First listed: {}",
      listed.lines().next().unwrap_or("<none>")
    );
  }
}
