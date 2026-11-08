//! End-to-end: `.sdx` to a running binary.
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`), leaf 7. Adapted from
//! `/data/Code/emerald/crates/emerald-cli/tests/hello_em.rs` — the harness
//! shape is Emerald's (locate the fixture via `CARGO_MANIFEST_DIR`, invoke the
//! genuinely-built binary via `CARGO_BIN_EXE_*`, assert exact stdout, clean
//! up), with one step Emerald has no need for: Redox emits Rust source, so the
//! harness invokes real `rustc` on the generated `.xrs` before running it.
//!
//! Plan 01 §2 measured that rustc compiles a `.xrs` file directly, with zero
//! stderr, settling doc 01 §1.1's open question about input-file extension
//! handling — so no `.rs` staging file is needed here.
//!
//! Why all three of text, compilation and execution are asserted: text can be
//! stable and wrong, and generated Rust can compile and still do the wrong
//! thing. Doc 02 §8.1-8.2 treats the distinction as load-bearing.

use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A unique scratch directory per test, so a parallel run cannot collide.
fn scratch(tag: &str) -> PathBuf {
  let dir = std::env::temp_dir().join(format!("redox-{}-{}", tag, std::process::id()));
  std::fs::create_dir_all(&dir).expect("should create a scratch directory");
  dir
}

/// Transpile `example`, returning the generated Rust and the `.xrs` path.
fn transpile(example: &str, dir: &Path) -> (String, PathBuf) {
  let source = workspace_root().join("examples").join(example);
  let xrs = dir.join(Path::new(example).with_extension("xrs"));

  let out = Command::new(env!("CARGO_BIN_EXE_redox"))
    .arg("build")
    .arg(&source)
    .arg("-o")
    .arg(&xrs)
    .output()
    .expect("should run the redox binary");

  assert!(
    out.status.success(),
    "redox build failed on {example}:\n{}",
    String::from_utf8_lossy(&out.stderr)
  );

  let rust = std::fs::read_to_string(&xrs).expect("should read the generated .xrs");
  (rust, xrs)
}

/// The toolchain's rustc, resolved at build time by `build.rs`. Resolving it
/// from `PATH` here is unreliable: under `blaze test` the test process sees
/// NixOS's `stub-ld` shim instead.
const RUSTC: &str = env!("REDOX_RUSTC");

/// Whether the baked rustc can actually be executed in this process's
/// environment.
fn rustc_runs() -> bool {
  Command::new(RUSTC)
    .arg("--version")
    .output()
    .is_ok_and(|o| o.status.success())
}

/// Announce a skip loudly. Used only when rustc genuinely cannot be executed,
/// never to paper over a compile failure — the compile-and-run proof is plan
/// 01's core acceptance criterion and `moon run :cargo-test`, which is in the
/// required gate, always has a working rustc.
fn skip_without_rustc(test: &str) -> bool {
  if rustc_runs() {
    return false;
  }
  eprintln!(
    "SKIP {test}: cannot execute rustc at {RUSTC}.\n\
     The generated-Rust proof is verified by `moon run :cargo-test`; see\n\
     AGENTS.md's known tooling gotchas for why `blaze test` cannot run it."
  );
  true
}

/// Compile `xrs` with real rustc and run it, returning its stdout.
fn compile_and_run(xrs: &Path, dir: &Path) -> String {
  let bin = dir.join("program");
  let out = Command::new(RUSTC)
    .arg("--edition")
    .arg("2024")
    .arg("-o")
    .arg(&bin)
    .arg(xrs)
    .output()
    .expect("should run rustc");

  // Acceptance criterion 3: zero stderr, not merely a zero exit code. A
  // warning is a defect in the emitter, because the user never wrote the
  // generated line it would point at.
  assert!(
    out.stderr.is_empty(),
    "rustc should emit nothing for generated code, got:\n{}",
    String::from_utf8_lossy(&out.stderr)
  );
  assert!(
    out.status.success(),
    "rustc should compile the generated .xrs"
  );

  let run = Command::new(&bin)
    .output()
    .expect("should run the compiled binary");
  assert!(run.status.success(), "the compiled binary should exit 0");
  String::from_utf8_lossy(&run.stdout).into_owned()
}

/// Plan 01 §3's concrete proof, end to end.
#[test]
fn hello_sdx_transpiles_compiles_and_prints_hello() {
  if skip_without_rustc("hello_sdx_transpiles_compiles_and_prints_hello") {
    return;
  }
  let dir = scratch("hello");
  let (rust, xrs) = transpile("hello.sdx", &dir);

  // The generated text is asserted by `tests/golden/hello.xrs` and the output
  // by `examples/hello.stdout`, both blessable. Plan 08 §2.1: this file held
  // the whole `.xrs` as an escaped literal, duplicating the golden in a form
  // only a human editing Rust could update.
  assert!(
    rust.contains("println!"),
    "the program should print:\n{rust}"
  );

  assert_eq!(compile_and_run(&xrs, &dir), "hello\n");
  std::fs::remove_dir_all(&dir).ok();
}

/// Acceptance criterion 5, end to end: trivia reaches the binary's source.
#[test]
fn comments_survive_into_the_generated_rust() {
  if skip_without_rustc("comments_survive_into_the_generated_rust") {
    return;
  }
  let dir = scratch("comments");
  let (rust, xrs) = transpile("comments.sdx", &dir);

  assert!(
    rust.contains("// Trivia must survive"),
    "a leading comment should survive:\n{rust}"
  );
  assert!(
    rust.contains("println!(\"one\"); // an inline comment"),
    "an inline comment should survive:\n{rust}"
  );
  assert!(
    rust.trim_end().ends_with("// A comment at end of file."),
    "a trailing comment should survive:\n{rust}"
  );

  assert_eq!(compile_and_run(&xrs, &dir), "one\ntwo\n");
  std::fs::remove_dir_all(&dir).ok();
}

/// Acceptance criterion 7: two runs over the same input are byte-identical.
#[test]
fn transpiling_twice_produces_identical_bytes() {
  let dir = scratch("determinism");
  let (first, _) = transpile("comments.sdx", &dir);
  let (second, _) = transpile("comments.sdx", &dir);
  assert_eq!(first, second);
  std::fs::remove_dir_all(&dir).ok();
}

/// Acceptance criterion 8, through the real binary: a tab is rejected with a
/// diagnostic naming the line.
#[test]
fn rejects_a_tab_in_indentation_with_a_located_diagnostic() {
  let dir = scratch("tab");
  let bad = dir.join("tab.sdx");
  std::fs::write(&bad, "main = ->\n\tprint \"x\"\n").expect("should write the fixture");

  let out = Command::new(env!("CARGO_BIN_EXE_redox"))
    .arg("build")
    .arg(&bad)
    .output()
    .expect("should run the redox binary");

  assert!(!out.status.success(), "a tab should be a build failure");
  let stderr = String::from_utf8_lossy(&out.stderr);
  assert!(stderr.contains("tab"), "{stderr}");
  assert!(stderr.contains("tab.sdx:2:1"), "{stderr}");
  std::fs::remove_dir_all(&dir).ok();
}

/// Acceptance criterion 9, through the real binary.
#[test]
fn rejects_an_inconsistent_dedent_with_a_located_diagnostic() {
  let dir = scratch("dedent");
  let bad = dir.join("dedent.sdx");
  std::fs::write(&bad, "main = ->\n    print \"a\"\n  print \"b\"\n")
    .expect("should write the fixture");

  let out = Command::new(env!("CARGO_BIN_EXE_redox"))
    .arg("build")
    .arg(&bad)
    .output()
    .expect("should run the redox binary");

  assert!(!out.status.success(), "an inconsistent dedent should fail");
  let stderr = String::from_utf8_lossy(&out.stderr);
  assert!(stderr.contains("inconsistent dedent"), "{stderr}");
  assert!(stderr.contains("dedent.sdx:3:"), "{stderr}");
  std::fs::remove_dir_all(&dir).ok();
}

/// The default output mapping of doc 01 §3.4, exercised for real.
#[test]
fn writes_a_sibling_xrs_when_no_output_is_given() {
  let dir = scratch("default-output");
  let src = dir.join("prog.sdx");
  std::fs::write(&src, "main = ->\n  print \"x\"\n").expect("should write the fixture");

  let status = Command::new(env!("CARGO_BIN_EXE_redox"))
    .arg("build")
    .arg(&src)
    .status()
    .expect("should run the redox binary");

  assert!(status.success());
  assert!(dir.join("prog.xrs").exists(), "prog.xrs should exist");
  std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn reports_a_version_and_a_usage_message() {
  for flag in ["--version", "--help"] {
    let out = Command::new(env!("CARGO_BIN_EXE_redox"))
      .arg(flag)
      .output()
      .expect("should run the redox binary");
    assert!(out.status.success(), "{flag} should succeed");
    assert!(!out.stdout.is_empty(), "{flag} should print something");
  }
}
