//! The **lint** corpus: programs that compile, and warn — plan 38.
//!
//! `tests/ui/` asserts the opposite invariant. Its harness says
//!
//! ```text
//! assert!(!out.status.success(), "a UI case must be rejected");
//! ```
//!
//! which is load-bearing: a case that was meant to fail and quietly starts
//! succeeding would otherwise pass. So a warning cannot live there, and plan
//! 38 §10 records discovering that — the plan had assumed the UI corpus was
//! severity-agnostic and it is an *error* corpus by construction.
//!
//! This corpus asserts the mirror invariant: the program **must build**, and
//! its `.stderr` is whatever Redox said while building it. The pair of
//! corpora now covers both halves the way `examples/` (must run, zero output)
//! and `tests/ui/` (must fail) already did.
//!
//! An expectation may be **empty**, and one is: `unwrap_suppressed.stderr`.
//! That is the only way to prove suppression works — a corpus entry that
//! asserts nothing was printed.
//!
//! To accept a deliberate change, run with `REDOX_BLESS=1`.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// Every `.sdx` under `tests/lint/`, sorted.
fn cases() -> Vec<PathBuf> {
  let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lint");
  let mut found: Vec<PathBuf> = std::fs::read_dir(&dir)
    .unwrap_or_else(|e| panic!("should read {}: {e}", dir.display()))
    .map(|e| e.expect("should read an entry").path())
    .filter(|p| p.extension().is_some_and(|x| x == "sdx"))
    .collect();
  found.sort();
  found
}

/// Run `redox build --plain` on one case and return what it printed.
///
/// The assertion is the inverse of the UI corpus's, and it is the point: a
/// lint that failed the build would be an error with a misleading label.
fn stderr_of(source: &Path) -> String {
  let out = Command::new(env!("CARGO_BIN_EXE_redox"))
    .arg("build")
    .arg(source)
    .arg("--plain")
    .arg("-o")
    .arg(std::env::temp_dir().join("redox-lint-discard.xrs"))
    .output()
    .expect("should run the redox binary");
  assert!(
    out.status.success(),
    "a lint case must build — a warning that fails the build is an error with the wrong \
     label. {} exited {:?}:\n{}",
    source.display(),
    out.status.code(),
    String::from_utf8_lossy(&out.stderr)
  );
  assert!(
    out.stdout.is_empty(),
    "a diagnostic belongs on stderr, not stdout: {}",
    String::from_utf8_lossy(&out.stdout)
  );

  let text = String::from_utf8_lossy(&out.stderr).to_string();
  let name = source.file_name().expect("a name").to_string_lossy();
  text.replace(&source.display().to_string(), &name)
}

#[test]
fn every_lint_case_matches_its_expectation() {
  let bless = std::env::var_os("REDOX_BLESS").is_some();
  let cases = cases();
  assert!(!cases.is_empty(), "the lint corpus should not be empty");

  let mut blessed = Vec::new();
  for source in &cases {
    let actual = stderr_of(source);
    let expected_path = source.with_extension("stderr");
    if bless {
      std::fs::write(&expected_path, &actual).expect("should write the expectation");
      blessed.push(expected_path.clone());
      continue;
    }
    let expected = std::fs::read_to_string(&expected_path).unwrap_or_else(|e| {
      panic!(
        "{} has no expectation ({e}); run the suite with REDOX_BLESS=1 to record one",
        source.display()
      )
    });
    assert_eq!(
      actual,
      expected,
      "{} printed something other than its expectation; re-run with REDOX_BLESS=1 if the \
       change was deliberate",
      source.display()
    );
  }

  assert!(
    blessed.is_empty() || bless,
    "blessing happened without REDOX_BLESS"
  );
}

/// A `.stderr` with no `.sdx` beside it is a stale expectation nothing reads —
/// the same guard `ui.rs` carries, for the same reason.
#[test]
fn every_expectation_has_a_case() {
  let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lint");
  for entry in std::fs::read_dir(&dir).expect("should read tests/lint") {
    let path = entry.expect("should read an entry").path();
    if path.extension().is_some_and(|x| x == "stderr") {
      assert!(
        path.with_extension("sdx").exists(),
        "{} has no case beside it",
        path.display()
      );
    }
  }
}
