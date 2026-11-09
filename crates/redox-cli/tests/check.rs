//! `redox check` puts **rustc's** errors on the user's `.sdx`.
//!
//! Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) leaf 6. These tests run the
//! rustc the toolchain ships, which is the standard of proof doc 02 §6.5 item
//! 4 set and the reason they are end-to-end rather than unit tests: the thing
//! being proved is that a real compiler's real output lands in the right
//! place, and a mocked rustc would prove only that the mock matches the
//! parser.
//!
//! The programs live in `tests/check/*.sdx`, not in literals here. Plan 07
//! wrote them inline and plan 08's `no_literal_expectations.rs` caught it —
//! AGENTS.md's fixture rule exists because Emerald's one syntax change could
//! not reach 113 fixtures hidden inside Rust test modules. Each fixture also
//! explains, in a comment, which case it is, which a literal could not do
//! without cluttering the assertion.
//!
//! Every line and column asserted below is written out as a literal, so a
//! reader can check the claim against the fixture without running anything.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

struct Checked {
  stderr: String,
  success: bool,
  dir: PathBuf,
}

impl Drop for Checked {
  fn drop(&mut self) {
    std::fs::remove_dir_all(&self.dir).ok();
  }
}

fn fixture(name: &str) -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("tests/check")
    .join(format!("{name}.sdx"))
}

/// Copy a fixture into a scratch directory and run `redox check --plain`.
///
/// Copied rather than checked in place so the diagnostic's path is short and
/// predictable in the assertions, and so nothing is written next to the
/// fixture.
fn check(name: &str) -> Checked {
  let dir = std::env::temp_dir().join(format!("redox-check-{name}-{}", std::process::id()));
  std::fs::create_dir_all(&dir).expect("should create a scratch directory");
  let sdx = dir.join(format!("{name}.sdx"));
  std::fs::copy(fixture(name), &sdx).expect("should copy the fixture");

  let out = Command::new(env!("CARGO_BIN_EXE_redox"))
    .arg("check")
    .arg(&sdx)
    .arg("--plain")
    .output()
    .expect("should run the redox binary");

  Checked {
    stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    success: out.status.success(),
    dir,
  }
}

/// Criteria 1 and 2: rustc's message and code, against the `.sdx`, on the
/// right line.
///
/// `tests/check/verbatim.sdx` has the mistake on **line 6**, at the `+` in
/// column 5 of `  a + "x"`.
#[test]
fn a_type_error_is_reported_against_the_sdx() {
  if support::skip_without_rustc("a_type_error_is_reported_against_the_sdx") {
    return;
  }
  let checked = check("verbatim");
  assert!(!checked.success, "a type error must fail the check");

  // rustc's code and message, verbatim — plan 07 §1.1.
  assert!(
    checked.stderr.contains("error[E0277]"),
    "rustc's own code should pass through:\n{}",
    checked.stderr
  );
  assert!(
    checked.stderr.contains("cannot add `&str` to `i32`"),
    "rustc's own message should pass through:\n{}",
    checked.stderr
  );

  assert!(
    checked.stderr.contains("verbatim.sdx:6:5"),
    "should point at verbatim.sdx line 6 column 5:\n{}",
    checked.stderr
  );
  assert!(
    !checked.stderr.contains(".xrs"),
    "the user should never be shown the generated file here:\n{}",
    checked.stderr
  );
}

/// Criterion 3: on a verbatim line the column is exact.
///
/// Source line 6 is `  a + "x"`; its `+` is at column 5. rustc reported column
/// 7 of `    a + "x"`, which is the same `+`. The shift is the difference in
/// indentation and nothing more.
#[test]
fn the_column_is_exact_on_a_verbatim_line() {
  if support::skip_without_rustc("the_column_is_exact_on_a_verbatim_line") {
    return;
  }
  let checked = check("verbatim");
  assert!(
    checked.stderr.contains("verbatim.sdx:6:5"),
    "{}",
    checked.stderr
  );
  // The caret sits under the `+`: the gutter, then the column-5 offset.
  assert!(
    checked.stderr.contains("  |     ^\n"),
    "the caret should be a single `^` under the `+`:\n{}",
    checked.stderr
  );
}

/// Criterion 4: on a restructured line the statement is underlined, not a
/// clamped column.
///
/// `tests/check/restructured.sdx` line 11 is `  print add("x", 2)` — 19
/// characters, 17 of them the statement. It becomes
/// `    println!("{}", add("x", 2));`, and rustc reports column 24. Shifting
/// that gives column 22, past the end of the source line, so the whole
/// statement is underlined instead (plan 07 §2.1). This test exists so that
/// behaviour cannot quietly become a clamp.
#[test]
fn a_restructured_line_underlines_the_statement() {
  if support::skip_without_rustc("a_restructured_line_underlines_the_statement") {
    return;
  }
  let checked = check("restructured");
  assert!(!checked.success);
  assert!(
    checked.stderr.contains("error[E0308]"),
    "{}",
    checked.stderr
  );
  assert!(
    checked.stderr.contains("restructured.sdx:11:3"),
    "should point at the start of the statement on line 11:\n{}",
    checked.stderr
  );
  assert!(
    checked.stderr.contains("^^^^^^^^^^^^^^^^^"),
    "the whole 17-character statement should be underlined:\n{}",
    checked.stderr
  );
}

/// Criterion 6: a program rustc accepts produces nothing and exits zero.
#[test]
fn a_clean_program_passes_silently() {
  if support::skip_without_rustc("a_clean_program_passes_silently") {
    return;
  }
  let checked = check("clean");
  assert!(
    checked.success,
    "a correct program should pass:\n{}",
    checked.stderr
  );
  assert_eq!(checked.stderr, "", "and should say nothing");
}

/// A syntax error is Redox's own and never reaches rustc: it keeps its `RDX`
/// code rather than acquiring a rustc one.
#[test]
fn a_syntax_error_keeps_its_redox_code() {
  let checked = check("syntax");
  assert!(!checked.success);
  assert!(
    checked.stderr.contains("error[RDX0102]"),
    "a parse error belongs to Redox:\n{}",
    checked.stderr
  );
}

/// Criterion 7: the `redox` binary finds a rustc of its own at run time, not
/// only under `cargo test`.
///
/// Asserted indirectly but unambiguously: if the binary had failed to resolve
/// rustc it would have said so, and that message is one only this path emits.
#[test]
fn the_binary_resolves_rustc_itself() {
  if support::skip_without_rustc("the_binary_resolves_rustc_itself") {
    return;
  }
  let checked = check("trivial");
  assert!(
    !checked.stderr.contains("no rustc found"),
    "the binary should resolve rustc at run time:\n{}",
    checked.stderr
  );
  assert!(checked.success);
}

/// `redox check` must not leave the `.xrs` it handed rustc lying around: the
/// user asked a question, not for a build artefact.
#[test]
fn check_leaves_no_artefacts_behind() {
  if support::skip_without_rustc("check_leaves_no_artefacts_behind") {
    return;
  }
  let dir = std::env::temp_dir().join(format!("redox-check-tidy-{}", std::process::id()));
  std::fs::create_dir_all(&dir).expect("should create");
  let sdx = dir.join("trivial.sdx");
  std::fs::copy(fixture("trivial"), &sdx).expect("should copy the fixture");

  let out = Command::new(env!("CARGO_BIN_EXE_redox"))
    .arg("check")
    .arg(&sdx)
    .output()
    .expect("should run");
  assert!(out.status.success());

  let left: Vec<PathBuf> = std::fs::read_dir(&dir)
    .expect("should list")
    .map(|e| e.expect("entry").path())
    .filter(|p| p != &sdx)
    .collect();
  assert!(
    left.is_empty(),
    "check should leave only the source behind, found {left:?}"
  );
  std::fs::remove_dir_all(&dir).ok();
}

/// Every fixture is referenced by a test above, and every test's fixture
/// exists. A fixture nothing reads is dead weight that still looks like
/// coverage.
#[test]
fn every_check_fixture_is_used() {
  let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/check");
  let this = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/check.rs");
  let source = std::fs::read_to_string(&this).expect("should read check.rs");
  for entry in std::fs::read_dir(&dir).expect("should read tests/check") {
    let path = entry.expect("should read an entry").path();
    let stem = path.file_stem().expect("a stem").to_string_lossy();
    assert!(
      source.contains(&format!("check(\"{stem}\")"))
        || source.contains(&format!("fixture(\"{stem}\")")),
      "{stem}.sdx is not used by any test in check.rs"
    );
  }
}
