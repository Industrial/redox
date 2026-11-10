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
    checked.stderr.contains("error[E0277]"),
    "{}",
    checked.stderr
  );
  assert!(
    checked.stderr.contains("restructured.sdx:16:3"),
    "should point at the start of the statement on line 16:\n{}",
    checked.stderr
  );
  assert!(
    checked.stderr.contains("^^^^^^^^^^^^^^^^^^^^^^^^^^"),
    "the whole 26-character statement should be underlined:\n{}",
    checked.stderr
  );
}

/// Plan 54 criterion 3: `:=` against a binding that is not `mut` is rustc's
/// error, reported against the `.sdx`.
///
/// This is the row's load-bearing negative. `is_mut` is set only by the
/// literal `mut` keyword (`parser.rs`) and the emitter writes `mut` only when
/// it is set (`emit/stmt.rs`), so Redox has no mechanism to make an immutable
/// assignment compile — and this test is what stops one being added as a
/// convenience later. *"Every check rustc can make, rustc makes."*
///
/// # Why this is in `tests/check/` and not `tests/ui/`
///
/// Plan 54 criterion 3 named `tests/ui/`, and it cannot live there: that
/// harness runs `redox build`, which parses, lints, emits and writes without
/// ever invoking rustc. A program whose only fault is `E0384` **builds
/// clean**, so `ui.rs`'s `assert!(!out.status.success())` would fail on its
/// own expectation rather than on the program. `tests/check/` is the corpus
/// that runs real rustc and remaps its spans, which is what this needs.
#[test]
fn assigning_to_an_immutable_binding_is_rustcs_error() {
  if support::skip_without_rustc("assigning_to_an_immutable_binding_is_rustcs_error") {
    return;
  }
  let checked = check("immutable_assignment");
  assert!(!checked.success);
  assert!(
    checked.stderr.contains("error[E0384]"),
    "rustc's immutable-assignment error, not a Redox one:\n{}",
    checked.stderr
  );
  assert!(
    checked.stderr.contains("immutable_assignment.sdx:19:3"),
    "reported against the `:=` on line 19 of the .sdx:\n{}",
    checked.stderr
  );
  assert!(
    !checked.stderr.contains(".xrs"),
    "never against the generated Rust:\n{}",
    checked.stderr
  );
}

/// Plan 55 criterion 7: a discarded `Result` is reported against the `.sdx`,
/// and the program still builds.
///
/// # Why this is the criterion's witness and not a `tests/lint/` case
///
/// Criterion 7 asks for *"a Redox lint, not only a rustc warning"*, and leaf 4
/// worded the value as **position** — the discard lint *"at the Redox line
/// rather than rustc's"*. The position is already here, by plan 07's
/// remapping, so there is no `RDX1004`: the fixture's own header argues it at
/// length, and the two statements are the argument.
///
/// `save 1` calls a function **this file declares** as returning
/// `Result<i32, String>`, which is the only discard a lexical lint could
/// recognise — the return type is in the source, so no types are needed.
/// `u8::try_from 300` is the shape that actually bites, and nothing in the
/// `.sdx` says it returns anything; recognising it needs the standard
/// library's signatures, which is the type checker AGENTS.md forbids Redox
/// growing. A lint restricted to the first would fire only where rustc
/// already fires and stay silent on the second, so its coverage would be a
/// strict subset of the warning it was meant to improve on.
///
/// `tests/lint/` could not hold this either way: its harness runs `redox
/// build`, which never invokes rustc, so the warning does not exist there.
/// `tests/check/` is the corpus that runs real rustc and remaps its spans.
///
/// `tests/check/discarded_result.sdx` discards on **lines 34 and 35**, both at
/// column 3.
#[test]
fn a_discarded_result_is_reported_against_the_sdx() {
  if support::skip_without_rustc("a_discarded_result_is_reported_against_the_sdx") {
    return;
  }
  let checked = check("discarded_result");
  assert!(
    checked.success,
    "a discard is legal, so the check must still pass:\n{}",
    checked.stderr
  );

  // rustc's own wording, which is the half plan 07 passes through verbatim.
  assert_eq!(
    checked
      .stderr
      .matches("unused `Result` that must be used")
      .count(),
    2,
    "one warning per discard:\n{}",
    checked.stderr
  );
  assert!(
    checked.stderr.contains("discarded_result.sdx:34:3"),
    "the locally-declared `Result`, on line 34:\n{}",
    checked.stderr
  );
  assert!(
    checked.stderr.contains("discarded_result.sdx:35:3"),
    "the `std` one a lexical lint could not see, on line 35:\n{}",
    checked.stderr
  );
  assert!(
    !checked.stderr.contains(".xrs"),
    "never against the generated Rust:\n{}",
    checked.stderr
  );
}

/// Plan 56 criterion 3: a borrowed capture produces a **rustc** error, at the
/// generated closure, reported against the `.sdx`.
///
/// This is the row's §1.3 claim being measured rather than argued: an
/// under-applied call allocates a closure over the arguments it was given, and
/// a capture that outlives its borrow is rustc's to reject. Redox has no borrow
/// checker and must never grow one, so if rustc did *not* catch this the
/// feature would be unsound and nothing else in the gate would notice.
///
/// # Why this is in `tests/check/` and not `tests/ui/`
///
/// Plan 56 criterion 3 names `tests/ui/`, and it cannot live there for exactly
/// the reason plan 54 criterion 3 could not — see
/// [`assigning_to_an_immutable_binding_is_rustcs_error`] above. `ui.rs` runs
/// `redox build`, which parses, lints, emits and writes without ever invoking
/// rustc, so this program **builds clean** and that harness's
/// `assert!(!out.status.success())` would fail on its own expectation rather
/// than on the program. Two plans have now named `tests/ui/` for a rustc
/// error; the corpus that runs real rustc and remaps its spans is this one.
///
/// `tests/check/borrowed_capture.sdx` under-applies on **line 20**, at the
/// argument paren in column 26 of
/// `make_prefixer = Box::new (prefix (s.as_str ()))`.
#[test]
fn a_borrowed_capture_is_rustcs_error_at_the_generated_closure() {
  if support::skip_without_rustc("a_borrowed_capture_is_rustcs_error_at_the_generated_closure") {
    return;
  }
  let checked = check("borrowed_capture");
  assert!(
    !checked.success,
    "a capture that outlives its borrow must fail"
  );

  // rustc's own code and wording. `E0373` and not `E0382`, which is what
  // `move` would have produced — plan 56 §5 criterion 6 withdrew the `move`
  // so that this is the message the author sees.
  assert!(
    checked.stderr.contains("error[E0373]"),
    "rustc's closure-outlives error, not a Redox one:\n{}",
    checked.stderr
  );
  assert!(
    checked.stderr.contains("but it borrows `s`"),
    "rustc should name the captured binding:\n{}",
    checked.stderr
  );
  assert!(
    checked.stderr.contains("borrowed_capture.sdx:20:26"),
    "reported against the under-applied call on line 20 of the .sdx:\n{}",
    checked.stderr
  );
  assert!(
    !checked.stderr.contains(".xrs"),
    "never against the generated Rust:\n{}",
    checked.stderr
  );
}

/// Plan 57 §11.4, as rustc's error rather than as a claim: **a field selector
/// moves.**
///
/// `?.` was deleted on §1.4's argument that the chain *"is expressible in the
/// ordinary combinator vocabulary"*. Probed, it is not — `(.tag)` lowers to
/// `|__rdx_1| __rdx_1.tag` with no `&`, so `map (.tag)` over a shared
/// reference is `E0507`, and the replacement in `examples/fallible.sdx`
/// therefore writes every `&` by hand. This is the test that says so, and the
/// third plan in a row to need `tests/check/` rather than `tests/ui/` for a
/// rustc error — `ui.rs` runs `redox build`, which never invokes rustc.
///
/// `Tag` is deliberately not `Clone`, exactly as `examples/fallible.sdx`
/// makes it, so a lowering that cloned would compile here instead.
#[test]
fn a_field_selector_moves_and_rustc_says_so() {
  if support::skip_without_rustc("a_field_selector_moves_and_rustc_says_so") {
    return;
  }
  let checked = check("moved_selector");
  assert!(
    !checked.success,
    "selecting a non-`Clone` field out of a borrow must fail"
  );
  assert!(
    checked.stderr.contains("error[E0507]"),
    "rustc's move-out-of-borrow error:\n{}",
    checked.stderr
  );
  // The generated closure's parameter, which is what names the lowering in
  // rustc's own words — and the reason the `&` cannot be left to the emitter.
  assert!(
    checked.stderr.contains("`__rdx_1.tag`"),
    "rustc should name the selector's own binding:\n{}",
    checked.stderr
  );
  assert!(
    checked.stderr.contains("moved_selector.sdx:25:35"),
    "reported against the section on line 25 of the .sdx:\n{}",
    checked.stderr
  );
  assert!(
    !checked.stderr.contains(".xrs"),
    "never against the generated Rust:\n{}",
    checked.stderr
  );
}

/// Plan 67 criterion 5: the program §1.4 measured at `E0049` is now silent.
///
/// `class Convert t` used to spend its one head slot on the implementing type,
/// so a second lowercase name in a member's type became a **method** generic —
/// `fn wrap<U>(…)` in the trait against `fn wrap(n: f64)` in the impl, which
/// rustc rejects by naming a type parameter the author never wrote. With the
/// receiver marked the slot is the trait's, and `trait Convert<T>` with two
/// impls for one type is reachable.
///
/// # Why this is in `tests/check/` and not `tests/ui/`
///
/// The **fourth** plan to need this, after 54, 56 and 57: the fault is rustc's,
/// and `ui.rs` runs `redox build`, which parses, lints and emits without ever
/// invoking rustc. A program whose only defect is `E0049` therefore *builds
/// clean*, and a `tests/ui/` fixture would fail on its own
/// `assert!(!status.success())` rather than on the program.
#[test]
fn a_generic_class_with_two_impls_for_one_type_is_accepted() {
  if support::skip_without_rustc("a_generic_class_with_two_impls_for_one_type_is_accepted") {
    return;
  }
  let checked = check("generic_class");
  assert!(
    checked.success,
    "`trait Convert<T>` with two impls for one type should compile:\n{}",
    checked.stderr
  );
  // No `E0049` in particular, and nothing at all in general — a warning would
  // be as much of a regression here as an error, because the old shape's
  // failure was a type parameter nobody wrote.
  assert_eq!(checked.stderr, "", "and should say nothing");
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
