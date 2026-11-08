//! `redox check` puts **rustc's** errors on the user's `.sdx`.
//!
//! Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) leaf 6. These tests run the
//! rustc the toolchain ships, which is the standard of proof doc 02 §6.5 item
//! 4 set and the reason they are end-to-end rather than unit tests: the thing
//! being proved is that a real compiler's real output lands in the right
//! place, and a mocked rustc would prove only that the mock matches the
//! parser.
//!
//! Every line and column this file asserts is written out as a literal, so a
//! reader can check the claim against the `.sdx` beside it without running
//! anything.

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

/// Write `src` to a `.sdx` and run `redox check --plain` over it.
fn check(tag: &str, src: &str) -> Checked {
  let dir = std::env::temp_dir().join(format!("redox-check-{tag}-{}", std::process::id()));
  std::fs::create_dir_all(&dir).expect("should create a scratch directory");
  let sdx = dir.join(format!("{tag}.sdx"));
  std::fs::write(&sdx, src).expect("should write the source");

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
/// ```text
/// 1  add = (a: i32, b: i32): i32 ->
/// 2    a + "x"                        <- the mistake
/// ```
///
/// The generated Rust has `a + "x"` on *its* line 2 as well, but that is a
/// coincidence of this program's shape, not the mechanism: the next test uses
/// a program where the two differ.
#[test]
fn a_type_error_is_reported_against_the_sdx() {
  let checked = check(
    "verbatim",
    "add = (a: i32, b: i32): i32 ->\n  a + \"x\"\n\nmain = ->\n  print add(1, 2)\n",
  );
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

  // Criterion 2: the position names the `.sdx`, line 2.
  assert!(
    checked.stderr.contains("verbatim.sdx:2:5"),
    "should point at verbatim.sdx line 2 column 5:\n{}",
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
/// Source line 2 is `  a + "x"`; its `+` is at column 5. rustc reported column
/// 7 of `    a + "x"`, which is the same `+`. The shift is the difference in
/// indentation and nothing more.
#[test]
fn the_column_is_exact_on_a_verbatim_line() {
  let checked = check(
    "exact",
    "add = (a: i32, b: i32): i32 ->\n  a + \"x\"\n\nmain = ->\n  print add(1, 2)\n",
  );
  assert!(
    checked.stderr.contains("exact.sdx:2:5"),
    "{}",
    checked.stderr
  );
  // The caret sits under the `+`: four spaces of gutter prefix, then the
  // column-5 offset.
  assert!(
    checked.stderr.contains("  |     ^\n"),
    "the caret should be a single `^` under the `+`:\n{}",
    checked.stderr
  );
}

/// Criterion 4: on a restructured line the statement is underlined, not a
/// clamped column.
///
/// ```text
/// 5    print add("x", 2)              <- 19 characters
/// ```
///
/// becomes `    println!("{}", add("x", 2));`, and rustc reports column 24.
/// Shifting that gives column 22 — past the end of the source line. The whole
/// statement is underlined instead (plan 07 §2.1), and this test exists so
/// that behaviour cannot quietly become a clamp.
#[test]
fn a_restructured_line_underlines_the_statement() {
  let checked = check(
    "restructured",
    "add = (a: i32, b: i32): i32 ->\n  a + b\n\nmain = ->\n  print add(\"x\", 2)\n",
  );
  assert!(!checked.success);
  assert!(
    checked.stderr.contains("error[E0308]"),
    "{}",
    checked.stderr
  );
  assert!(
    checked.stderr.contains("restructured.sdx:5:3"),
    "should point at the start of the statement on line 5:\n{}",
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
  let checked = check(
    "clean",
    "add = (a: i32, b: i32): i32 ->\n  a + b\n\nmain = ->\n  print add(1, 2)\n",
  );
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
  let checked = check("syntax", "main = ->\n  print 1 +\n");
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
/// rustc it would have said so, and the message is one only this path emits.
#[test]
fn the_binary_resolves_rustc_itself() {
  let checked = check("resolve", "main = ->\n  print 1\n");
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
  let dir = std::env::temp_dir().join(format!("redox-check-tidy-{}", std::process::id()));
  std::fs::create_dir_all(&dir).expect("should create");
  let sdx = dir.join("tidy.sdx");
  std::fs::write(&sdx, "main = ->\n  print 1\n").expect("should write");

  let out = Command::new(env!("CARGO_BIN_EXE_redox"))
    .arg("check")
    .arg(&sdx)
    .output()
    .expect("should run");
  assert!(out.status.success());

  let left: Vec<PathBuf> = std::fs::read_dir(&dir)
    .expect("should list")
    .map(|e| e.expect("entry").path())
    .filter(|p| p != Path::new(&sdx))
    .collect();
  assert!(
    left.is_empty(),
    "check should leave only the source behind, found {left:?}"
  );
  std::fs::remove_dir_all(&dir).ok();
}
