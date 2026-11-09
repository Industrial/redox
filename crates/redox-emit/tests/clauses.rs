//! Doc 18 §2.3's multi-clause equations.
//!
//! A function is a signature line plus **one or more** clauses, each a row of
//! patterns and a body. Several clauses, or one refutable pattern, lower to a
//! single `fn` whose body is a `match`.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// Doc 18 §2.3's own example, and the lowering it specifies.
#[test]
fn three_clauses_become_one_match() {
  let out = xrs("fib :: u64 -> u64\nfib 0 =\n  0\nfib 1 =\n  1\nfib n =\n  fib (n - 1)\n");
  assert!(out.contains("fn fib(__rdx_a0: u64) -> u64 {"), "{out}");
  assert!(out.contains("match __rdx_a0 {"), "{out}");
  assert!(out.contains("0 => {"), "{out}");
  assert!(out.contains("1 => {"), "{out}");
  assert!(out.contains("n => {"), "{out}");
  assert_eq!(out.matches("fn fib").count(), 1, "one function, not three");
}

/// **The property that keeps doc 18 §1's rule true.** A single clause whose
/// patterns all bind is the ordinary function doc 03 had, and lowers to
/// exactly the Rust it lowered to before clauses existed — no `match`, no
/// generated parameter name. This is why the goldens did not move when §2.3
/// arrived.
#[test]
fn one_irrefutable_clause_emits_no_match() {
  let out = xrs("add :: i32 -> i32 -> i32\nadd a b =\n  a + b\n");
  assert_eq!(
    out, "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    "{out}"
  );
}

/// A binding pattern lends its name to the parameter; a testing pattern
/// cannot, so the parameter takes a generated name and the pattern moves into
/// the arm.
#[test]
fn a_testing_pattern_gets_a_generated_parameter_name() {
  let out = xrs("f :: u32 -> u32\nf 0 =\n  1\nf n =\n  n\n");
  assert!(out.contains("fn f(__rdx_a0: u32)"), "{out}");
  assert!(
    !out.contains("fn f(0"),
    "a literal cannot be a parameter name:\n{out}"
  );
}

/// Several parameters match as a tuple, which is Rust's own way of matching
/// more than one value and needs no temporary.
#[test]
fn several_parameters_match_as_a_tuple() {
  let out = xrs("g :: u32 -> u32 -> u32\ng 0 0 =\n  0\ng a b =\n  a + b\n");
  assert!(out.contains("match (__rdx_a0, __rdx_a1) {"), "{out}");
  assert!(out.contains("(0, 0) => {"), "{out}");
  assert!(out.contains("(a, b) => {"), "{out}");
}

/// `_` matches and discards, and is irrefutable — so on its own it needs no
/// `match` either.
#[test]
fn a_wildcard_is_irrefutable() {
  let out = xrs("f :: u32 -> u32\nf _ =\n  1\n");
  assert!(!out.contains("match"), "a lone `_` needs no match:\n{out}");
}

/// A one-clause function with a refutable pattern still gets a `match`,
/// because the pattern has to be tested somewhere. rustc then reports the
/// non-exhaustiveness, which is correct: doc 18 §1 keeps rustc as the
/// authority on exhaustiveness and Redox must never grow a checker for it.
#[test]
fn one_refutable_clause_still_matches() {
  let out = xrs("f :: u32 -> u32\nf 0 =\n  1\n");
  assert!(out.contains("match __rdx_a0 {"), "{out}");
  assert!(out.contains("0 => {"), "{out}");
}

/// A literal pattern is carried verbatim, the same discipline doc 03 §14.1
/// applies to types — so Redox never has to agree with rustc about what a
/// literal means.
#[test]
fn a_literal_pattern_is_verbatim() {
  let out = xrs("f :: u8 -> u8\nf 0xFF =\n  1\nf n =\n  n\n");
  assert!(out.contains("0xFF => {"), "{out}");
}
