//! What a Voltaic program **prints when it runs**, kept in files beside it.
//!
//! Plan 08 (`docs/13 - 08 - Run Expectations.md`) leaf 1. The third corpus:
//! `tests/golden/` asserts the generated text, `tests/ui/` asserts the
//! diagnostic, and this asserts the behaviour.
//!
//! A run expectation lives **beside its program**, under the same stem:
//!
//! ```text
//! examples/phase1.sdx         examples/phase1.stdout
//! tests/run/strings.sdx       tests/run/strings.stdout
//! ```
//!
//! Two directories, one rule. `examples/` holds the programs a reader should
//! see — AGENTS.md calls them the CI-checked source of truth for what works
//! today — and `tests/run/` holds cases too narrow to earn a place there, like
//! one lexical hazard per line. Putting the expectation beside the program
//! rather than in a parallel tree is what keeps the two from drifting, and is
//! why `examples/expressions.sdx` is not duplicated here: it already *is* the
//! precedence case.
//!
//! Every case is compiled by **real rustc** and executed. Doc 02 §8.1–8.2 is
//! the reason: generated text can be byte-stable and still mean the wrong
//! thing, and plan 03 proved it — `a + b;` looked right and evaluated to
//! `()`. Precedence is the sharpest case, because `2 + 3 * 4` has a correct
//! answer, and `14` distinguishes a right tree from a wrong one better than
//! reading a dump ever will.
//!
//! `VOLTAIC_UPDATE=1` re-records every `.stdout` from what the programs actually
//! printed, so a deliberate change is reviewed as a diff. Without it a
//! mismatch fails.
//!
//! A case whose output declares doc 03 §15's `#[cfg(test)]` module is **also
//! compiled as a test binary and run**. An ordinary build does not compile
//! that module at all, so until this was added the corpus checked nothing
//! about the generated tests: not that they compile, not that they pass, and
//! not that two descriptions had slugged to the same function name.

mod support;

use std::path::{Path, PathBuf};

/// Examples that cannot run, with the reason.
///
/// Mirrors `examples.rs`'s `NOT_PARSEABLE`: `lexical.sdx` exercises the lexer
/// with constructs the parser does not accept yet, so it has no behaviour to
/// assert. Listed rather than inferred from a missing `.stdout`, because
/// inferring it would make *forgetting* an expectation indistinguishable from
/// declaring one unnecessary.
const CANNOT_RUN: &[&str] = &["lexical.sdx"];

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn corpus() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/run")
}

fn sdx_in(dir: &Path) -> Vec<PathBuf> {
  let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
    .unwrap_or_else(|e| panic!("should read {}: {e}", dir.display()))
    .map(|e| e.expect("should read an entry").path())
    .filter(|p| p.extension().is_some_and(|x| x == "sdx"))
    .collect();
  found.sort();
  found
}

/// Every program with a run expectation, from both directories.
fn cases() -> Vec<PathBuf> {
  let mut all = sdx_in(&workspace_root().join("examples"));
  all.retain(|p| {
    let name = p.file_name().expect("a name").to_string_lossy().to_string();
    !CANNOT_RUN.contains(&name.as_str())
  });
  all.extend(sdx_in(&corpus()));
  all
}

fn label(source: &Path) -> String {
  source.file_name().expect("a name").to_string_lossy().into()
}

/// Criteria 1 and 2: every case prints exactly what it is recorded as
/// printing.
#[test]
fn every_run_case_prints_its_expectation() {
  if support::skip_without_rustc("every_run_case_prints_its_expectation") {
    return;
  }
  let updating = std::env::var_os("VOLTAIC_UPDATE").is_some();
  let cases = cases();
  assert!(!cases.is_empty(), "the run corpus should not be empty");

  for source in &cases {
    let name = label(source);
    let case = support::transpile_path(source);
    // Doc 03 §15's test module is `#[cfg(test)]`, so the binary above never
    // compiles it. A case that declares tests has them compiled and run
    // here — without this the corpus checked nothing at all about them.
    case.run_tests();
    let actual = case.run();
    let expected_path = source.with_extension("stdout");

    if updating {
      std::fs::write(&expected_path, &actual).expect("should write the expectation");
      continue;
    }
    let expected = std::fs::read_to_string(&expected_path).unwrap_or_else(|e| {
      panic!("{name} has no .stdout ({e}); run with VOLTAIC_UPDATE=1 to record one")
    });
    assert_eq!(
      actual, expected,
      "{name} printed something other than its expectation; \
       re-run with VOLTAIC_UPDATE=1 if the change was deliberate"
    );
  }
}

/// Criterion 3: no half-pairs, in either direction.
///
/// A `.stdout` with no `.sdx` is a stale expectation nothing reads. A runnable
/// program with no `.stdout` is coverage someone meant to add and did not.
///
/// Skipped under `VOLTAIC_UPDATE`, because updating is when the expectations are
/// written and this would race the test writing them.
#[test]
fn every_case_is_a_complete_pair() {
  if std::env::var_os("VOLTAIC_UPDATE").is_some() {
    eprintln!("skipping every_case_is_a_complete_pair: updating creates the pairs");
    return;
  }

  for source in cases() {
    assert!(
      source.with_extension("stdout").exists(),
      "{} has no .stdout beside it; add one with VOLTAIC_UPDATE=1, or list the \
       program in CANNOT_RUN with a reason",
      label(&source)
    );
  }

  for dir in [workspace_root().join("examples"), corpus()] {
    for entry in std::fs::read_dir(&dir).expect("should read the directory") {
      let path = entry.expect("should read an entry").path();
      if path.extension().is_some_and(|x| x == "stdout") {
        assert!(
          path.with_extension("sdx").exists(),
          "{} has no .sdx beside it",
          path.display()
        );
      }
    }
  }
}

/// The corpus directory holds nothing but pairs, so a stray file cannot sit
/// there being ignored.
#[test]
fn the_corpus_holds_only_cases_and_expectations() {
  for entry in std::fs::read_dir(corpus()).expect("should read the corpus") {
    let path = entry.expect("should read an entry").path();
    let ext = path.extension().and_then(|x| x.to_str());
    assert!(
      matches!(ext, Some("sdx" | "stdout")),
      "{} is neither a case nor an expectation",
      path.display()
    );
  }
}

/// Criterion 4: each case explains itself in the program, and the explanation
/// reaches the generated Rust.
///
/// A `.stdout` cannot hold a comment, so the reasoning — why `2 + 3 * 4` is
/// `14` and not `20` — lives in the `.sdx`. Asserting it survives into the
/// `.xrs` exercises trivia preservation (doc 02 §10.1) on every case, free.
#[test]
fn every_case_documents_itself_and_keeps_its_comments() {
  if support::skip_without_rustc("every_case_documents_itself_and_keeps_its_comments") {
    return;
  }
  for source in cases() {
    let name = label(&source);
    let text = std::fs::read_to_string(&source).expect("should read the case");
    assert!(
      // `--`, not `#`: doc 18 §5.1 took Haskell's comment syntax, which is
      // what retired four of doc 03 §4.1's six meanings of `#`.
      text.lines().any(|l| l.trim_start().starts_with("--")),
      "{name} should explain in a comment what it is demonstrating"
    );
    assert!(
      support::transpile_path(&source).rust.contains("//"),
      "{name}'s comments should reach the generated Rust (doc 02 §10.1)"
    );
  }
}
