//! Shared helpers for the end-to-end tests.
//!
//! A `mod support;` include rather than a construct file, because an
//! integration test's helpers are not part of the crate's vocabulary — doc 06
//! §7 exempts `tests/` from the one-construct rule for exactly this reason.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// The toolchain's rustc, resolved at build time by `build.rs`. Resolving it
/// from `PATH` here is unreliable: under `blaze test` the test process sees
/// NixOS's `stub-ld` shim instead.
const RUSTC: &str = env!("REDOX_RUSTC");

pub struct Case {
  pub rust: String,
  xrs: PathBuf,
  dir: PathBuf,
}

impl Case {
  /// Compile the generated Rust with real rustc and run it, returning stdout.
  ///
  /// Asserts **zero stderr**, not merely a zero exit code: a warning is a
  /// defect in the emitter, because the user never wrote the generated line it
  /// would point at.
  pub fn run(&self) -> String {
    let bin = self.dir.join("program");
    let out = Command::new(RUSTC)
      .args(["--edition", "2024", "-o"])
      .arg(&bin)
      .arg(&self.xrs)
      .output()
      .expect("should run rustc");
    assert!(
      out.stderr.is_empty(),
      "rustc should emit nothing for generated code, got:\n{}",
      String::from_utf8_lossy(&out.stderr)
    );
    assert!(
      out.status.success(),
      "rustc should compile the generated .xrs"
    );

    let run = Command::new(&bin).output().expect("should run the binary");
    assert!(run.status.success(), "the compiled binary should exit 0");
    String::from_utf8_lossy(&run.stdout).into_owned()
  }
}

impl Drop for Case {
  fn drop(&mut self) {
    std::fs::remove_dir_all(&self.dir).ok();
  }
}

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scratch(tag: &str) -> PathBuf {
  let dir = std::env::temp_dir().join(format!("redox-{tag}-{}", std::process::id()));
  std::fs::create_dir_all(&dir).expect("should create a scratch directory");
  dir
}

/// Transpile an `examples/` fixture.
pub fn transpile(example: &str) -> Case {
  let source = workspace_root().join("examples").join(example);
  build(
    &source,
    Path::new(example)
      .file_stem()
      .expect("a stem")
      .to_str()
      .unwrap(),
  )
}

/// Transpile inline source. Used only where a fixture would exist solely to
/// pin one grammar rule; anything a reader should be able to run lives in
/// `examples/` as a file (doc 06 §7).
pub fn transpile_source(tag: &str, src: &str) -> String {
  let dir = scratch(tag);
  let sdx = dir.join(format!("{tag}.sdx"));
  std::fs::write(&sdx, src).expect("should write the source");
  let case = build(&sdx, tag);
  let rust = case.rust.clone();
  rust
}

fn build(source: &Path, tag: &str) -> Case {
  let dir = scratch(tag);
  let xrs = dir.join(format!("{tag}.xrs"));
  let out = Command::new(env!("CARGO_BIN_EXE_redox"))
    .arg("build")
    .arg(source)
    .arg("-o")
    .arg(&xrs)
    .output()
    .expect("should run the redox binary");
  assert!(
    out.status.success(),
    "redox build failed on {}:\n{}",
    source.display(),
    String::from_utf8_lossy(&out.stderr)
  );
  let rust = std::fs::read_to_string(&xrs).expect("should read the generated .xrs");
  Case { rust, xrs, dir }
}
