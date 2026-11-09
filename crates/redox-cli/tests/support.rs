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
  /// The source map `redox build` wrote beside the `.xrs` (plan 06).
  pub fn map_path(&self) -> PathBuf {
    let mut name = self.xrs.file_name().expect("a name").to_os_string();
    name.push(".map");
    self.xrs.with_file_name(name)
  }

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

  /// Compile the generated Rust **as a test binary** and run its tests.
  ///
  /// Doc 03 §15's test module is `#[cfg(test)]`, so an ordinary build does
  /// not compile it at all — which means the corpus checked *nothing* about
  /// the generated tests until this existed: not that they compile, not that
  /// they pass, not that two slugs collided into one function name.
  ///
  /// A no-op for a file with no test module, so every case can call it.
  pub fn run_tests(&self) {
    let rust = std::fs::read_to_string(&self.xrs).expect("should read the .xrs");
    if !rust.contains("#[cfg(test)]") {
      return;
    }

    let bin = self.dir.join("tests");
    let out = Command::new(RUSTC)
      .args(["--edition", "2024", "--test", "-o"])
      .arg(&bin)
      .arg(&self.xrs)
      .output()
      .expect("should run rustc");
    assert!(
      out.stderr.is_empty(),
      "rustc should emit nothing for the generated test module, got:\n{}",
      String::from_utf8_lossy(&out.stderr)
    );
    assert!(
      out.status.success(),
      "rustc should compile the generated test module"
    );

    let run = Command::new(&bin).output().expect("should run the tests");
    assert!(
      run.status.success(),
      "the generated tests should pass:\n{}",
      String::from_utf8_lossy(&run.stdout)
    );
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

/// A rustfmt that behaves like rustfmt.
///
/// Verified rather than trusted, the same rule `build.rs` applies to rustc —
/// plan 03's third blaze shim is why that rule exists.
fn rustfmt_path() -> Option<PathBuf> {
  let dir = Path::new(RUSTC).parent()?;
  let candidate = dir.join("rustfmt");
  let works = Command::new(&candidate)
    .arg("--version")
    .output()
    .is_ok_and(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).contains("rustfmt"));
  works.then_some(candidate)
}

pub fn rustfmt_available() -> bool {
  rustfmt_path().is_some()
}

/// Format `rust` with rustfmt's **defaults**, outside the repository.
///
/// This repo's own `rustfmt.toml` sets two-space indentation, which governs
/// Redox's *source*. Redox's *output* uses rustfmt's default four, so running
/// rustfmt inside the repo would grade the output against the wrong config —
/// a real finding from plan 04's probe, where every example "failed" purely on
/// indentation until the check moved out of the tree.
pub fn rustfmt(rust: &str) -> String {
  let fmt = rustfmt_path().expect("caller should check rustfmt_available");
  // `std::env::temp_dir()` is outside the repository, so no `rustfmt.toml`
  // applies and rustfmt uses its defaults.
  let dir = scratch("rustfmt");
  let file = dir.join("generated.rs");
  std::fs::write(&file, rust).expect("should write the generated Rust");

  let out = Command::new(&fmt)
    .args(["--edition", "2024"])
    .arg(&file)
    .output()
    .expect("should run rustfmt");
  assert!(
    out.status.success(),
    "rustfmt failed on generated code:\n{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let formatted = std::fs::read_to_string(&file).expect("should read the formatted file");
  std::fs::remove_dir_all(&dir).ok();
  formatted
}

/// A scratch directory unique to this call.
///
/// Keyed by a counter as well as the pid: keying on the tag alone let two
/// concurrent tests share a directory, so one test's `Case::drop` deleted the
/// `.xrs` another was still compiling. Found by running the suite, not by
/// reading it.
fn scratch(tag: &str) -> PathBuf {
  use std::sync::atomic::{AtomicUsize, Ordering};
  static NEXT: AtomicUsize = AtomicUsize::new(0);
  let n = NEXT.fetch_add(1, Ordering::Relaxed);
  let dir = std::env::temp_dir().join(format!("redox-{tag}-{}-{n}", std::process::id()));
  std::fs::create_dir_all(&dir).expect("should create a scratch directory");
  dir
}

/// Transpile an `examples/` fixture.
pub fn transpile(example: &str) -> Case {
  transpile_path(&workspace_root().join("examples").join(example))
}

/// Transpile the `.sdx` at `source`, wherever it lives.
///
/// Plan 08 (`docs/13 - 08 - Run Expectations.md`) leaf 1: the run corpus is
/// under `tests/run/`, not `examples/`, so it needs a path rather than a name.
pub fn transpile_path(source: &Path) -> Case {
  let tag = source
    .file_stem()
    .expect("a stem")
    .to_str()
    .expect("a UTF-8 stem");
  build(source, tag)
}

/// The verified rustc path, for handing to a child process.
pub fn rustc() -> &'static str {
  RUSTC
}

/// Whether a rustc that actually runs is available.
///
/// Verified by execution, not by the path existing — plan 01 found NixOS's
/// `stub-ld` shim answering to `rustc` under `blaze test`, which fails only
/// when invoked.
pub fn rustc_runs() -> bool {
  Command::new(RUSTC)
    .arg("--version")
    .output()
    .is_ok_and(|o| o.status.success())
}

/// Skip `test` — reporting that it was skipped — when no rustc will run.
///
/// Returns `true` if the caller should return early. A silent skip is worse
/// than no test, so the reason is printed.
pub fn skip_without_rustc(test: &str) -> bool {
  if rustc_runs() {
    return false;
  }
  eprintln!("skipping {test}: no working rustc at {RUSTC}");
  true
}

/// A cargo that actually runs, for the tests that build a nested project.
///
/// **Not** `env!("CARGO")` alone. Under `blaze test` that is
/// `~/.blaze/toolchains/…/bin/cargo`, a generic-linux binary NixOS refuses to
/// execute:
///
/// ```text
/// Could not start dynamically linked executable: …/bin/cargo
/// ```
///
/// which is the fourth time a toolchain binary has answered to the right name
/// and not worked (plan 01's `stub-ld` rustc, plan 03's two blaze shims, now
/// this). So the rule AGENTS.md records is applied again: try the candidates
/// in order and **check the banner**, never trust the name.
pub fn cargo_path() -> Option<PathBuf> {
  let from_env = std::env::var_os("CARGO").map(PathBuf::from);
  let baked = PathBuf::from(env!("CARGO"));
  let from_path = std::env::var_os("PATH").into_iter().flat_map(|paths| {
    std::env::split_paths(&paths)
      .map(|d| d.join("cargo"))
      .collect::<Vec<_>>()
  });

  from_env
    .into_iter()
    .chain(std::iter::once(baked))
    .chain(from_path)
    .find(|c| {
      Command::new(c).arg("--version").output().is_ok_and(|o| {
        o.status.success() && String::from_utf8_lossy(&o.stdout).starts_with("cargo ")
      })
    })
}

/// Skip `test` when no cargo will run.
pub fn skip_without_cargo(test: &str) -> bool {
  if cargo_path().is_some() {
    return false;
  }
  eprintln!("skipping {test}: no working cargo (blaze's is not executable on NixOS)");
  true
}

/// Transpile inline source. Used only where a fixture would exist solely to
/// pin one grammar rule; anything a reader should be able to run lives in
/// `examples/` as a file (doc 06 §7).
pub fn transpile_source(tag: &str, src: &str) -> String {
  let dir = scratch(tag);
  let sdx = dir.join(format!("{tag}.sdx"));
  std::fs::write(&sdx, src).expect("should write the source");
  // Cloned rather than moved: `Case::drop` deletes the scratch directory, so
  // the field cannot be moved out of it.
  build(&sdx, tag).rust.clone()
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
  // **Redox's own diagnostics count too, as of plan 38.** The corpus has
  // always demanded zero *rustc* output; it said nothing about Redox's,
  // because until `RDX1001` every Redox diagnostic was fatal and
  // `status.success()` covered them all.
  //
  // A warning broke that: `examples/fallible.sdx` emitted three `RDX1001`s
  // that nothing noticed. An example is exemplary code, so a lint firing on
  // one is either a bug in the example or a deliberate exception that should
  // say so with the pragma. Plan 38 §10 records it.
  assert!(
    out.stderr.is_empty(),
    "redox should emit nothing for an example, got:\n{}\n\
     if a warning is intended, suppress it at the call site and say why",
    String::from_utf8_lossy(&out.stderr)
  );
  let rust = std::fs::read_to_string(&xrs).expect("should read the generated .xrs");
  Case { rust, xrs, dir }
}
