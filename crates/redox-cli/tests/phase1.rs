//! Plan 03's concrete proof: doc 01 §31 Phase 1's own acceptance test, which
//! plan 01 deliberately deferred.
//!
//! `docs/08 - 03 - Parser Core.md` §3. Text, compilation and execution are all
//! asserted, per doc 02 §8.1–8.2 — text can be stable and wrong, and generated
//! Rust can compile and still do the wrong thing.
//!
//! **Where those assertions now live.** Plan 08 moved two of the three out of
//! this file and into corpora, because they were string literals here and a
//! literal expectation can only be updated by a human editing Rust:
//!
//! | Assertion | Now in |
//! |---|---|
//! | the generated text | `tests/golden/phase1.xrs` |
//! | what it prints | `examples/phase1.stdout` |
//! | the shapes below | here |
//!
//! What remains is what a corpus cannot express: claims about the *shape* of
//! generated Rust, where the point is that one substring appears and no output
//! is being compared. Those are not expectations-as-data, they are assertions
//! about a grammar rule, and they belong in code.

mod support;

/// Criterion 4: a type round-trips verbatim, however nested.
#[test]
fn a_nested_generic_type_round_trips() {
  let rust = support::transpile_source(
    "nested_generic",
    "f : Vec<HashMap<String, u8>> -> usize\nf m =\n  m.len ()\n",
  );
  assert!(
    rust.contains("m: Vec<HashMap<String, u8>>"),
    "the type should survive verbatim:\n{rust}"
  );
  assert!(rust.contains("-> usize"), "{rust}");
}

/// Criterion 5: the `->`-after-`)` refinement of plan 03 §1.1. Without it,
/// `&dyn Fn(i32) -> i32` cannot be a parameter type and doc 03 §14.1's "every
/// Rust type form is reachable" would be false.
#[test]
fn a_function_type_parameter_parses_as_one_type() {
  let rust = support::transpile_source(
    "fn_type",
    "apply : &dyn Fn(i32) -> i32 -> i32 -> i32\napply f x =\n  f x\n",
  );
  assert!(
    rust.contains("f: &dyn Fn(i32) -> i32"),
    "the function type should be one parameter:\n{rust}"
  );
  assert!(
    rust.contains("x: i32"),
    "the second parameter should survive"
  );
}
