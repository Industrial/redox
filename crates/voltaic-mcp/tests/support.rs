//! Fixtures for the MCP tests.
//!
//! Plan 50. AGENTS.md: *"Every `.sdx` fixture is a file, and tests reference
//! files."* These tests need files on disk — a tool takes a **path** — so each
//! case copies a committed fixture from `tests/fixtures/` into a scratch
//! directory and works on the copy. Nothing here writes into the repository,
//! and `voltaic_build` writes an `.xrs` beside its input.

use std::path::{Path, PathBuf};

/// A scratch copy of a committed fixture.
pub struct Case {
  dir: PathBuf,
  pub sdx: PathBuf,
}

impl Case {
  /// Copy `tests/fixtures/<name>.sdx` into a fresh scratch directory.
  ///
  /// `temp_dir()` rather than the repository tree, for `examples.rs`'s reason:
  /// the repository's own tooling config applies inside it.
  pub fn new(name: &str) -> Self {
    let dir = std::env::temp_dir().join(format!(
      "voltaic-mcp-{name}-{}-{:?}",
      std::process::id(),
      std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("should create a scratch directory");

    let from = fixtures().join(format!("{name}.sdx"));
    let sdx = dir.join(format!("{name}.sdx"));
    std::fs::copy(&from, &sdx).unwrap_or_else(|e| panic!("should copy {}: {e}", from.display()));
    Self { dir, sdx }
  }

  pub fn path(&self) -> String {
    self.sdx.display().to_string()
  }

  pub fn sibling(&self, name: &str) -> PathBuf {
    self.dir.join(name)
  }
}

impl Drop for Case {
  fn drop(&mut self) {
    std::fs::remove_dir_all(&self.dir).ok();
  }
}

pub fn fixtures() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Whether a rustc that actually runs is available.
///
/// `voltaic_check` runs rustc, and under `blaze test` every candidate visible to
/// a test process is a shim — AGENTS.md's gotcha. The driver's `rustc_path`
/// verifies by execution, so this asks it and then asks the answer to run.
pub fn rustc_runs() -> bool {
  voltaic_driver::rustc_path().is_some_and(|rustc| {
    std::process::Command::new(rustc)
      .arg("--version")
      .output()
      .is_ok_and(|o| o.status.success())
  })
}

/// Skip `test` — saying so — when no rustc will run.
///
/// A silent skip is worse than no test (plan 46 §1.3), so the reason prints.
pub fn skip_without_rustc(test: &str) -> bool {
  if rustc_runs() {
    return false;
  }
  eprintln!("SKIP {test}: no rustc answered `--version` like rustc");
  true
}
