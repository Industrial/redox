//! Expectations are files, not string literals in Rust tests.
//!
//! Plan 08 (`docs/13 - 08 - Run Expectations.md`) leaf 4. AGENTS.md states the
//! rule for fixtures:
//!
//! > **Every `.sdx` fixture is a file, and tests reference files.** Never embed
//! > language source as a string literal in a Rust test. Emerald's one syntax
//! > cutover missed 113 fixtures that lived as string literals inside test
//! > modules; that cost is paid once per syntax change, forever.
//!
//! The rule had been applied to *sources* and not to *expectations*, and plan
//! 08 §2.1 found the gap: `phase1.rs` held the whole generated `.xrs` as an
//! escaped literal, duplicating `tests/golden/phase1.xrs`, plus a ten-line
//! `concat!` of expected output. Both were updatable only by hand, and one of
//! them could disagree with the blessable copy of the same fact.
//!
//! This test exists so the gap cannot reopen, because it already closed once
//! by accident and reopened without anyone noticing.
//!
//! **The check is a heuristic and says so.** A precise one would have to parse
//! Rust. This looks for a multi-line string literal that contains generated
//! Rust (`fn ` followed by a brace) or looks like captured program output —
//! and it accepts that a determined author can evade it. The point is to make
//! the easy path the right one, not to be unfoolable.

use std::path::{Path, PathBuf};

/// Files allowed to contain what the heuristic flags, with the reason.
///
/// Deliberately short and deliberately annotated: an allow with no reason is
/// how a check stops meaning anything.
const ALLOWED: &[(&str, &str)] = &[
  (
    "no_literal_expectations.rs",
    "this file quotes the patterns it forbids, in its own documentation",
  ),
  (
    "status.rs",
    "its literals are the TEMPLATE for a generated document, not an \
     expectation of program output. Plan 08 §2.1's rule is that an \
     expectation must be a file, and this test's expectation is \
     docs/STATUS.md — a file, compared byte for byte and blessable. The \
     heuristic sees long prose in a string and cannot tell a generator from \
     a captured stdout (plan 47 §10.2)",
  ),
];

fn tests_dir() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

fn test_files() -> Vec<PathBuf> {
  let mut found: Vec<PathBuf> = std::fs::read_dir(tests_dir())
    .expect("tests/ should exist")
    .map(|e| e.expect("should read an entry").path())
    .filter(|p| p.extension().is_some_and(|x| x == "rs"))
    .collect();
  found.sort();
  found
}

/// Strip `//` and `//!` comment lines: prose about a rule is not a violation
/// of it, and flagging documentation is how a check gets switched off.
fn code_only(text: &str) -> String {
  text
    .lines()
    .filter(|l| {
      let t = l.trim_start();
      !t.starts_with("//")
    })
    .collect::<Vec<_>>()
    .join("\n")
}

/// A literal holding generated Rust: `fn ` and a brace inside one `"…"`.
fn holds_generated_rust(code: &str) -> bool {
  code.contains("\\nfn ") || code.contains("fn main() {\\n") || code.contains("\\n}\\n\\n")
}

/// A literal holding captured output: three or more `\n`-separated values in
/// one literal, which is what a `concat!` of expected lines looks like.
fn holds_captured_output(code: &str) -> bool {
  code.matches("\\n\",").count() >= 3
}

#[test]
fn no_test_embeds_a_generated_rust_expectation() {
  let mut offenders = Vec::new();
  for path in test_files() {
    let name = path
      .file_name()
      .expect("a name")
      .to_string_lossy()
      .to_string();
    if ALLOWED.iter().any(|(f, _)| *f == name) {
      continue;
    }
    let code = code_only(&std::fs::read_to_string(&path).expect("should read"));
    if holds_generated_rust(&code) {
      offenders.push(format!(
        "  {name} embeds generated Rust as a string literal; \
         the golden corpus (tests/golden/) is where that belongs"
      ));
    }
  }
  assert!(
    offenders.is_empty(),
    "AGENTS.md requires expectations to be files (plan 08 §2.1):\n{}",
    offenders.join("\n")
  );
}

#[test]
fn no_test_embeds_a_captured_output_expectation() {
  let mut offenders = Vec::new();
  for path in test_files() {
    let name = path
      .file_name()
      .expect("a name")
      .to_string_lossy()
      .to_string();
    if ALLOWED.iter().any(|(f, _)| *f == name) {
      continue;
    }
    let code = code_only(&std::fs::read_to_string(&path).expect("should read"));
    if holds_captured_output(&code) {
      offenders.push(format!(
        "  {name} embeds program output as a string literal; \
         put the program beside a .stdout file instead"
      ));
    }
  }
  assert!(
    offenders.is_empty(),
    "AGENTS.md requires expectations to be files (plan 08 §2.1):\n{}",
    offenders.join("\n")
  );
}

/// Every entry in [`ALLOWED`] must name a file that exists, or the list
/// accumulates exemptions for files nobody can find.
#[test]
fn every_allowance_names_a_real_file() {
  for (name, why) in ALLOWED {
    assert!(
      tests_dir().join(name).exists(),
      "{name} is allowed ({why}) but does not exist"
    );
  }
}

/// The heuristic has to actually fire, or the two tests above pass by being
/// blind rather than by the tree being clean.
#[test]
fn the_heuristic_detects_what_it_claims_to() {
  let rust_literal = r#"assert_eq!(case.rust, "fn main() {\n    println!();\n}\n");"#;
  assert!(
    holds_generated_rust(rust_literal),
    "should flag an embedded .xrs"
  );

  let output_literal = r#"concat!("14\n", "5\n", "20\n",)"#;
  assert!(
    holds_captured_output(output_literal),
    "should flag embedded output"
  );

  let innocent = r#"assert!(rust.contains("-> usize"), "{rust}");"#;
  assert!(
    !holds_generated_rust(innocent) && !holds_captured_output(innocent),
    "a substring assertion about shape is not an expectation-as-data"
  );
}
