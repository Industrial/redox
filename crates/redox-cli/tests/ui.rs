//! The UI test corpus: every rejected program, with the exact bytes Redox
//! prints about it.
//!
//! Plan 05 (`docs/10 - 05 - Diagnostics.md`) leaf 5. A golden test asserts
//! that a *good* program lowers to the right Rust; this asserts that a *bad*
//! one is rejected with the right words, which is the half of a compiler's
//! behaviour that otherwise goes unmeasured. rustc keeps the same corpus for
//! the same reason.
//!
//! Each case is a pair under `tests/ui/`:
//!
//! ```text
//! tests/ui/empty_body.sdx       the program
//! tests/ui/empty_body.stderr    what `redox build --plain` prints
//! ```
//!
//! `--plain` rather than the graphical renderer because a byte comparison must
//! not depend on terminal width, colour support or `miette`'s version.
//!
//! To accept a deliberate change, run the suite with `REDOX_UPDATE=1`: every
//! `.stderr` is rewritten from the current output, and `git diff` then shows
//! precisely which messages moved. Without it a mismatch fails, so an
//! *accidental* change cannot pass quietly.

mod support;

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// Every `.sdx` under `tests/ui/`, sorted so failures are reported in a
/// predictable order.
fn cases() -> Vec<PathBuf> {
  let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ui");
  let mut found: Vec<PathBuf> = std::fs::read_dir(&dir)
    .unwrap_or_else(|e| panic!("should read {}: {e}", dir.display()))
    .map(|e| e.expect("should read an entry").path())
    .filter(|p| p.extension().is_some_and(|x| x == "sdx"))
    .collect();
  found.sort();
  found
}

/// Run `redox build --plain` on one case and return what it printed.
fn stderr_of(source: &Path) -> String {
  let out = Command::new(env!("CARGO_BIN_EXE_redox"))
    .arg("build")
    .arg(source)
    .arg("--plain")
    .arg("-o")
    .arg(std::env::temp_dir().join("redox-ui-discard.xrs"))
    .output()
    .expect("should run the redox binary");
  assert!(
    !out.status.success(),
    "a UI case must be rejected, but {} built successfully",
    source.display()
  );
  assert!(
    out.stdout.is_empty(),
    "a diagnostic belongs on stderr, not stdout: {}",
    String::from_utf8_lossy(&out.stdout)
  );

  // The case's own path appears in the `-->` line. Reduce it to the bare file
  // name so the expectation does not bake in this checkout's directory.
  let text = String::from_utf8_lossy(&out.stderr).to_string();
  let name = source.file_name().expect("a name").to_string_lossy();
  text.replace(&source.display().to_string(), &name)
}

/// Criterion 5: every UI case prints exactly its recorded diagnostic.
#[test]
fn every_ui_case_matches_its_expectation() {
  let updating = std::env::var_os("REDOX_UPDATE").is_some();
  let cases = cases();
  assert!(!cases.is_empty(), "the UI corpus should not be empty");

  let mut updated = Vec::new();
  for source in &cases {
    let actual = stderr_of(source);
    let expected_path = source.with_extension("stderr");
    if updating {
      std::fs::write(&expected_path, &actual).expect("should write the expectation");
      updated.push(expected_path.clone());
      continue;
    }
    let expected = std::fs::read_to_string(&expected_path).unwrap_or_else(|e| {
      panic!(
        "{} has no expectation ({e}); run the suite with REDOX_UPDATE=1 to record one",
        source.display()
      )
    });
    assert_eq!(
      actual,
      expected,
      "{} printed something other than its expectation; \
       re-run with REDOX_UPDATE=1 if the change was deliberate",
      source.display()
    );
  }
  assert!(
    updated.is_empty() || updating,
    "an expectation was rewritten without REDOX_UPDATE"
  );
}

/// A `.stderr` with no `.sdx` beside it is a case someone deleted and a
/// stale expectation left behind — which would otherwise never be noticed,
/// because nothing reads it.
#[test]
fn no_expectation_is_orphaned() {
  let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ui");
  for entry in std::fs::read_dir(&dir).expect("should read tests/ui") {
    let path = entry.expect("should read an entry").path();
    if path.extension().is_some_and(|x| x == "stderr") {
      assert!(
        path.with_extension("sdx").exists(),
        "{} has no .sdx beside it",
        path.display()
      );
    }
  }
}

/// Criterion 6: every diagnostic carries a code, and the code is the one the
/// corpus records. A message may be reworded; a code may not silently move.
#[test]
fn every_diagnostic_names_a_code() {
  for source in cases() {
    let text = stderr_of(&source);
    for line in text.lines().filter(|l| l.starts_with("error")) {
      let code = line
        .strip_prefix("error[")
        .and_then(|rest| rest.split(']').next())
        .unwrap_or_else(|| panic!("{} printed an uncoded error: {line}", source.display()));
      assert!(
        code.starts_with("RDX") && code.len() == 7,
        "{} printed a malformed code `{code}`",
        source.display()
      );
    }
  }
}
