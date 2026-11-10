//! The compiler reproduces itself: `stage1 == stage2`.
//!
//! Plan 72 leaf 4, and the assertion that makes a binary stage0 safe to trust.
//!
//! # Why this stops being optional once the seed is a blob
//!
//! While each ported crate's generated Rust is **committed**, a reviewer can
//! read the stage0 — it is 6,397 lines of ordinary Rust, and `tests/port.rs`
//! grades it against the `.sdx`. Once that tree is untracked and the stage0 is
//! `seed/redox1.wasm`, nobody can read it. `tests/blob.rs` then says the blob
//! is the one `SEED.md` claims, which is provenance and not behaviour.
//!
//! What is left is the oldest check in self-hosting: run the compiler, build
//! the compiler it produces, run *that*, and require the two outputs to be
//! identical. CoffeeScript's `cake build:full` is this — it builds twice so
//! the compiler compiles itself with the latest changes — and rustc's
//! stage1/stage2 and GHC's are the same idea at a larger scale.
//!
//! # What the three-way comparison actually proves
//!
//! Three artifacts go in:
//!
//! 1. `seed/redox1.wasm`, the committed stage0, run under `wasmtime`;
//! 2. the **native** `redox-seed`, which cargo compiled from
//!    `crates/redox-ast/src` and `crates/redox-fmt/src`;
//! 3. those same committed trees.
//!
//! The native binary is therefore *built from* the committed Rust. So asserting
//! `seed(sdx) == native(sdx) == committed` says: the stage0 emits exactly the
//! sources the stage1 compiler was built from, and the stage1 compiler emits
//! exactly the same sources again. That is `stage1 == stage2` — reached without
//! a second cargo build, because the equality on the third artifact is what
//! closes the loop.
//!
//! It would **not** hold if the committed trees and the seed's output had
//! merely been checked separately. The chain needs all three in one assertion,
//! which is why this is not two tests.
//!
//! # What it does not prove
//!
//! That the blob is reproducible from its recipe. `SEED.md` records the
//! command and `devenv.nix` pins `wasm-opt` and rustc, but a toolchain bump
//! moves the bytes legitimately. This is about behaviour, `blob.rs` is about
//! identity, and neither is about reproducibility.

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

/// rustfmt every file under `dir`, with **this repository's** configuration.
///
/// Not optional and not cosmetic: Redox emits for rustfmt's defaults and this
/// repository sets `tab_spaces = 2`, so the committed Rust is the formatted
/// form. `skip_children` because each file is formatted on its own and rustfmt
/// would otherwise try to resolve `mod` declarations that are not there yet.
fn format_tree(dir: &Path) {
  for rel in rust_files(dir) {
    let status = Command::new("rustfmt")
      .args(["--edition", "2024", "--config", "skip_children=true"])
      .arg(dir.join(&rel))
      .status()
      .expect("should run rustfmt — devenv.nix pins it");
    assert!(status.success(), "rustfmt should format {}", rel.display());
  }
}

/// Transpile `sdx` into a fresh directory under `target/`, and format it.
fn generate(label: &str, sdx: &Path, run: impl FnOnce(&Path, &Path)) -> PathBuf {
  let out = workspace_root().join("target").join(label);
  let _ = std::fs::remove_dir_all(&out);
  std::fs::create_dir_all(&out).expect("should create the output directory");
  run(sdx, &out);
  format_tree(&out);
  out
}

#[test]
fn stage1_equals_stage2() {
  let root = workspace_root();
  let sdx = root.join("crates/redox-ast/sdx");
  let committed = root.join("crates/redox-ast/src");
  let blob = root.join("seed/redox1.wasm");

  // ---------------------------------------------------------------- stage0
  let from_seed = generate("redox-fixpoint-stage0", &sdx, |sdx, out| {
    // **Relative to the preopen, not absolute.** `--dir .` grants exactly one
    // directory, and an absolute host path reaches nothing through it — the
    // seed sees no `.sdx`, writes nothing, and used to exit 0 while doing it.
    // That is how this test found `run.rs`'s missing empty-output check.
    let rel = |p: &Path| {
      p.strip_prefix(&root)
        .expect("every path here is under the workspace root")
        .to_path_buf()
    };
    let run = Command::new("wasmtime")
      .arg("run")
      // WASI grants no filesystem access at all without a preopen, so the
      // seed cannot read one `.sdx` without this. It is not a sandbox
      // convenience, it is the whole interface.
      .args(["--dir", "."])
      .arg(rel(&blob))
      .arg(rel(sdx))
      .arg(rel(out))
      .current_dir(&root)
      .output()
      .expect("should run wasmtime — devenv.nix pins it");
    assert!(
      run.status.success(),
      "the committed seed should transpile redox-ast's source:\n{}",
      String::from_utf8_lossy(&run.stderr)
    );
  });

  // ---------------------------------------------------------------- stage1
  let from_native = generate("redox-fixpoint-stage1", &sdx, |sdx, out| {
    let run = Command::new(env!("CARGO_BIN_EXE_redox-seed"))
      .arg(sdx)
      .arg(out)
      .output()
      .expect("should run the native redox-seed");
    assert!(
      run.status.success(),
      "the native seed should transpile redox-ast's source:\n{}",
      String::from_utf8_lossy(&run.stderr)
    );
  });

  // ------------------------------------------------------------- the loop
  let seed_files = rust_files(&from_seed);
  let native_files = rust_files(&from_native);
  let committed_files = rust_files(&committed);
  assert_eq!(
    seed_files, committed_files,
    "the committed seed produces a different set of files than is committed"
  );
  assert_eq!(
    native_files, committed_files,
    "the native seed produces a different set of files than is committed"
  );

  for rel in &committed_files {
    let stage0 = std::fs::read_to_string(from_seed.join(rel)).expect("should read stage0 output");
    let stage1 = std::fs::read_to_string(from_native.join(rel)).expect("should read stage1 output");
    let want = std::fs::read_to_string(committed.join(rel)).expect("should read committed Rust");

    assert_eq!(
      stage0,
      want,
      "{} — the **committed seed** does not emit what is committed. On a clean clone the \
       seed's output is all there is, so this is a crate that would not compile.",
      rel.display()
    );
    assert_eq!(
      stage1,
      stage0,
      "{} — stage1 and stage2 disagree. The compiler built from the seed's output does \
       not reproduce that output, so the seed and the tree have diverged and one of them \
       is lying about which compiler made it.",
      rel.display()
    );
  }
}
