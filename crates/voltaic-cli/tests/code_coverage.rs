//! Every diagnostic code must be reachable, and the UI corpus must reach it.
//!
//! Plan 05 (`docs/10 - 05 - Diagnostics.md`) leaf 6, and the test that gives
//! the code catalogue its teeth. A catalogue is a promise: `RDX0007` means
//! "unterminated attribute" and will go on meaning that. Two ways a promise
//! like this rots, both caught here:
//!
//! 1. A code is declared and never raised — so it means nothing, and nobody
//!    discovers that until a user greps for it.
//! 2. A code is raised but nothing pins what it prints, so a refactor can
//!    change which error a program gets and no test objects.
//!
//! The corpus is the evidence for both: a code is covered when some
//! `tests/ui/*.stderr` **or** `tests/lint/*.stderr` contains it. Declaring a code therefore obliges the
//! author to write the program that triggers it, which is the cheapest moment
//! to find out it cannot be triggered at all.

use std::collections::BTreeSet;
use std::path::Path;
use voltaic_diag::Code;

/// Codes whose only trigger is not reachable from the language Voltaic accepts
/// *today*, with the plan that will reach them. Every entry is a debt, so the
/// list is deliberately awkward to grow: the alternative is a code catalogue
/// nobody believes.
const NOT_YET_REACHABLE: &[(Code, &str)] = &[];

fn codes_in_corpus() -> BTreeSet<String> {
  let root = Path::new(env!("CARGO_MANIFEST_DIR"));
  let mut found = BTreeSet::new();
  // Two corpora since plan 38, because they assert opposite things: a UI case
  // must fail to build and a lint case must succeed. A warning has no home in
  // the first, and giving it one would have cost `ui.rs` the assertion that
  // makes it useful.
  for name in ["tests/ui", "tests/lint"] {
    let dir = root.join(name);
    for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("should read {name}: {e}")) {
      let path = entry.expect("should read an entry").path();
      if !path.extension().is_some_and(|x| x == "stderr") {
        continue;
      }
      let text = std::fs::read_to_string(&path).expect("should read an expectation");
      for line in text.lines() {
        // **Both severities.** A lint renders `warning[RDX1001]`, and a
        // catalogue that counted only errors would call its own first lint
        // unreachable.
        let rest = line
          .strip_prefix("error[")
          .or_else(|| line.strip_prefix("warning["));
        if let Some(rest) = rest
          && let Some(code) = rest.split(']').next()
        {
          found.insert(code.to_string());
        }
      }
    }
  }
  found
}

/// Criterion 6: every code the catalogue declares is reached by a UI case.
#[test]
fn every_code_is_reached_by_the_corpus() {
  let covered = codes_in_corpus();
  let excused: BTreeSet<String> = NOT_YET_REACHABLE.iter().map(|(c, _)| c.text()).collect();

  let missing: Vec<String> = Code::all()
    .iter()
    .map(|c| c.text())
    .filter(|t| !covered.contains(t) && !excused.contains(t))
    .collect();

  assert!(
    missing.is_empty(),
    "these codes are declared but no UI case reaches them: {}\n\
     Add a `crates/voltaic-cli/tests/ui/<name>.sdx` that triggers each, or list \
     it in NOT_YET_REACHABLE with the plan that will.",
    missing.join(", ")
  );
}

/// The excuse list must not outlive its excuse: a code listed there and *also*
/// covered means the debt was paid and the entry forgotten.
#[test]
fn no_excused_code_is_actually_reachable() {
  let covered = codes_in_corpus();
  for (code, why) in NOT_YET_REACHABLE {
    assert!(
      !covered.contains(&code.text()),
      "{} is listed as not-yet-reachable ({why}) but a UI case reaches it; \
       remove the entry",
      code.text()
    );
  }
}

/// A `.stderr` naming a code the catalogue does not declare means the code
/// came from somewhere other than `Code` — a hand-written string, most
/// likely, which is exactly what plan 05 set out to abolish.
#[test]
fn the_corpus_names_no_undeclared_code() {
  let declared: BTreeSet<String> = Code::all().iter().map(|c| c.text()).collect();
  for code in codes_in_corpus() {
    assert!(
      declared.contains(&code),
      "the corpus contains {code}, which `voltaic_diag::Code` does not declare"
    );
  }
}
