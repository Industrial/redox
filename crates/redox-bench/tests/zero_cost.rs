//! Plan 44 leaf 1: doc 03's zero-cost claim, asserted exactly.
//!
//! For every pair under `benchmarks/pairs/`, the generated Rust and the
//! hand-written Rust compile to **identical optimised LLVM IR**.
//!
//! # Why this is not a timing comparison
//!
//! Doc 04's row 44 asks for *"within noise"*, and §1.2 declines it for the
//! gate. A timing assertion is a flaky gate: plan 23 §10.8, doc 30 §8.7 and
//! plan 43 §10.3 are three rows that wrote *unexplained* into their results
//! because a gate failed for a load-dependent reason, and this repository's
//! blaze targets have been SIGKILLed by memory pressure from outside it.
//!
//! Identical IR is deterministic, exact and actionable: *within noise* becomes
//! *identical*, which is the claim doc 03 makes, and a failure is a diff
//! saying **what was added** rather than a number saying *slower*.
//!
//! # The negative case is the point of `unequal`
//!
//! A comparison that passes on every pair cannot distinguish *the lowering is
//! free* from *the comparison is vacuous*. `unequal.rs` clones where its
//! `.sdx` does not, and `the_comparison_can_fail` asserts the IR **differs**.
//!
//! That is not a hypothetical worry. `unequal` was the pair that exposed it:
//! its function was named after neither half's convention, so the comparison
//! looked for a symbol that was not there and found an empty body — which
//! passes an equality assertion for the worst possible reason.

use redox_bench::{Pair, function_ir, pairs, rustc_path};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scratch() -> PathBuf {
  root().join(".tmp/zero-cost")
}

/// The pair whose two sides are deliberately not equivalent.
const NEGATIVE: &str = "unequal";

/// Transpile a pair's `.sdx` to Rust.
///
/// Through the parser and emitter rather than the `redox` binary, so a failure
/// is a diagnostic here instead of an exit code, and so the test does not
/// depend on a build having happened.
fn generated(pair: &Pair) -> String {
  let source = std::fs::read_to_string(&pair.sdx)
    .unwrap_or_else(|e| panic!("should read {}: {e}", pair.sdx.display()));
  let program = redox_parser::parse_source(&source)
    .unwrap_or_else(|e| panic!("{} should parse: {e:?}", pair.sdx.display()));
  redox_emit::emit(&program)
}

fn hand_written(pair: &Pair) -> String {
  std::fs::read_to_string(&pair.rust)
    .unwrap_or_else(|e| panic!("should read {}: {e}", pair.rust.display()))
}

/// The emitted Rust with every item made `pub`, so `--crate-type=lib` keeps
/// it.
///
/// Redox emits a private `fn` for an unexported declaration, and a private
/// function in a library is dead code that rustc optimises away — which would
/// make every comparison trivially equal, and equal for the one reason that
/// means nothing. The hand-written halves are already `pub`; this is the
/// generated half catching up.
///
/// A rewrite of the *input to rustc*, not of the emitter's output as a
/// product: the visibility is irrelevant to the claim and necessary to measure
/// it.
fn as_library(rust: &str) -> String {
  rust
    .lines()
    .map(|line| {
      if line.starts_with("fn ") || line.starts_with("struct ") {
        format!("pub {line}")
      } else if line.starts_with("    ") && line.trim_end().ends_with(',') && line.contains(": ") {
        // A struct's fields need `pub` too, or the hand-written half's
        // `inner.x` has nothing to read.
        format!("    pub {}", line.trim_start())
      } else {
        line.to_string()
      }
    })
    .collect::<Vec<_>>()
    .join("\n")
}

/// Criterion 1: the corpus exists and every pair has both halves.
#[test]
fn every_pair_has_both_halves() {
  let found = pairs(&root());
  assert!(
    found.len() >= 6,
    "plan 44 §1.3 names seven lowerings; found {} pairs",
    found.len()
  );
  for pair in &found {
    assert!(pair.sdx.is_file(), "{} is missing", pair.sdx.display());
    assert!(pair.rust.is_file(), "{} is missing", pair.rust.display());
  }
  assert!(
    found.iter().any(|pair| pair.stem == NEGATIVE),
    "the negative case {NEGATIVE} should be in the corpus"
  );
}

/// Criterion 2: both halves of a pair define the **same** function.
///
/// The comparison reads a `define` block by name, so a pair whose two files
/// named their function differently would look for a symbol that is not there,
/// find an empty body, and pass every equality assertion vacuously.
///
/// The name is discovered from the hand-written half rather than derived from
/// the filename — `catch` and `guard` are keywords, so no function can be
/// called either — and this is the check that the `.sdx` agrees.
#[test]
fn both_halves_define_the_same_function() {
  let found = pairs(&root());
  assert!(!found.is_empty(), "the corpus should not be empty");
  for pair in found {
    let sdx = std::fs::read_to_string(&pair.sdx).expect("should read");
    assert!(
      sdx.contains(&format!("{} ::", pair.function)),
      "{} should declare `{} ::`, which is what {} defines",
      pair.sdx.display(),
      pair.function,
      pair.rust.display()
    );
  }
}

/// Criteria 3 and 5: identical optimised IR, for every equivalent pair.
///
/// Deterministic by construction — the same bytes every run — so this belongs
/// in the gate rather than in a report.
#[test]
fn the_lowerings_are_free() {
  assert!(
    rustc_path().is_some(),
    "no rustc answered `--version` like rustc; plan 44 §1.1 expects one, and \
     AGENTS.md records why neither PATH nor RUSTC can be trusted here"
  );

  let mut checked = 0;
  for pair in pairs(&root()) {
    if pair.stem == NEGATIVE {
      continue;
    }
    let generated_ir = function_ir(&as_library(&generated(&pair)), &pair.function, &scratch())
      .unwrap_or_else(|| {
        panic!(
          "the Rust generated from {} did not compile under -O",
          pair.sdx.display()
        )
      });
    let hand_ir = function_ir(&hand_written(&pair), &pair.function, &scratch())
      .unwrap_or_else(|| panic!("{} did not compile under -O", pair.rust.display()));

    assert!(
      !generated_ir.is_empty(),
      "`{}`'s comparison found no body for `{}`, which would pass equality \
       for the one reason that means nothing",
      pair.stem,
      pair.function
    );
    assert_eq!(
      generated_ir, hand_ir,
      "`{}` is not zero-cost: the generated Rust and the hand-written Rust \
       compile to different IR.\n\n--- generated\n{generated_ir}\n--- hand-written\n{hand_ir}",
      pair.stem
    );
    checked += 1;
  }
  assert!(
    checked >= 5,
    "only {checked} pairs were compared, which is too few to mean anything"
  );
}

/// Criterion 4: the comparison can fail.
///
/// Without this, `the_lowerings_are_free` proves only that it found nothing —
/// the same output whether the lowerings are free or the harness is comparing
/// empty strings.
#[test]
fn the_comparison_can_fail() {
  assert!(rustc_path().is_some(), "no verified rustc");
  let pair = pairs(&root())
    .into_iter()
    .find(|pair| pair.stem == NEGATIVE)
    .expect("the negative pair should exist");

  let generated_ir = function_ir(&as_library(&generated(&pair)), &pair.function, &scratch())
    .expect("the generated side should compile");
  let hand_ir = function_ir(&hand_written(&pair), &pair.function, &scratch())
    .expect("the hand side should compile");

  assert!(
    !generated_ir.is_empty(),
    "the comparison reads a function body, and this one is empty — which \
     would make every other assertion vacuous"
  );
  assert_ne!(
    generated_ir, hand_ir,
    "`{NEGATIVE}`'s two sides are deliberately not equivalent, so their IR \
     must differ — if it does not, the comparison is not comparing anything"
  );
}
