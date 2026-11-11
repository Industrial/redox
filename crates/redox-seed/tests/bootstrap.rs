//! A clone with no generated Rust can be bootstrapped from the committed seed.
//!
//! Plan 72 leaf 5, and the precondition that row set itself: *"never before CI
//! has rebuilt that crate from the seed on a clean clone."* There is no CI
//! configuration in this repository — the gate is the pre-push hook — so the
//! clean-clone proof is a test, like `status.rs` and `plan_orientation.rs`.
//!
//! # What it does
//!
//! Copies the **tracked working tree** into `target/`, runs `./bootstrap.sh`
//! there, and requires every file to come back byte-identical to what this
//! tree has.
//!
//! # Why the working tree and not `git clone HEAD`
//!
//! It cloned `HEAD` at first, which validated the **committed** seed against
//! the **working tree's** generated Rust. Those are two different compilers
//! in precisely the commit that matters most — the one refreshing the seed —
//! and `preCommitTargets` runs `:test`, so the result was a deadlock rather
//! than a warning: the seed could not be committed because this test failed,
//! and it could not pass until the seed was committed.
//!
//! The property leaf 5 wants is *"a checkout of **this** source, with the
//! generated Rust absent, bootstraps"*. That is the working tree.
//! `git ls-files` enumerates exactly the tracked set at its current content,
//! and the generated trees are untracked — so they are absent for free, and
//! the old delete-them-first step is gone with the reason for it.
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

/// The crates whose `src/` is generated, read from `seed/PORTED`.
///
/// **Not a constant, because this copy was one of the stale ones.** The list
/// lived in four places — here, `bootstrap.sh`, `port.rs` and a sentence in
/// `conformance.rs` — and this one named two crates when three were ported.
/// That is the worst place for it to go stale: `the_generated_trees_are_ignored`
/// is the assertion that a generated tree is never committed, and a crate
/// missing from the list is simply not asserted about. The check and the thing
/// it checks drifted together, which is what a duplicated list does.
fn ported() -> Vec<String> {
  let path = workspace_root().join("seed/PORTED");
  std::fs::read_to_string(&path)
    .unwrap_or_else(|e| panic!("should read {}: {e}", path.display()))
    .lines()
    .map(str::trim)
    .filter(|l| !l.is_empty() && !l.starts_with('#'))
    .filter_map(|l| l.split_whitespace().next().map(str::to_string))
    .collect()
}

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

  let listed = Command::new("git")
    .arg("ls-files")
    .current_dir(&root)
    .output()
    .expect("should run git ls-files");
  assert!(listed.status.success(), "git ls-files should succeed");
  let tracked = String::from_utf8_lossy(&listed.stdout);

  let mut copied = 0usize;
  for rel in tracked.lines().filter(|l| !l.trim().is_empty()) {
    let from = root.join(rel);
    // A tracked path that is not on disk is a staged deletion, which a
    // checkout of this state would not have either.
    if !from.is_file() {
      continue;
    }
    let to = clone.join(rel);
    if let Some(parent) = to.parent() {
      std::fs::create_dir_all(parent).expect("should create a directory in the copy");
    }
    std::fs::copy(&from, &to).unwrap_or_else(|e| panic!("should copy {rel}: {e}"));
    copied += 1;
  }
  assert!(
    copied > 200,
    "only {copied} tracked files copied, which is too few to be this repository — \
     the copy is the thing under test, so a broken copy must not read as a pass"
  );

  // The generated trees are untracked, so a copy of the tracked set does not
  // contain them. Asserted rather than assumed: if the ignore flip were ever
  // reverted they would arrive here and the bootstrap would be graded against
  // files it did not write.
  let crates = ported();
  for crate_name in &crates {
    let generated = clone.join("crates").join(crate_name).join("src");
    assert!(
      !generated.exists(),
      "{} is tracked, so it arrived in the copy — the ignore flip has been reverted \
       and this test would grade the bootstrap against files it did not produce",
      generated.display()
    );
  }

  // `std::fs::copy` preserves the permission bits, so the copied
  // `bootstrap.sh` is still executable.
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

  let crates = ported();
  for crate_name in &crates {
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
  let crates = ported();
  for crate_name in &crates {
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
