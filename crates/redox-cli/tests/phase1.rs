//! Plan 03's concrete proof: doc 01 §31 Phase 1's own acceptance test, which
//! plan 01 deliberately deferred.
//!
//! `docs/08 - 03 - Parser Core.md` §3. Text, compilation and execution are all
//! asserted, per doc 02 §8.1-8.2 — text can be stable and wrong, and generated
//! Rust can compile and still do the wrong thing.

mod support;

/// Criterion 1.
#[test]
fn phase1_transpiles_compiles_and_prints_30() {
  let case = support::transpile("phase1.sdx");
  assert_eq!(
    case.rust,
    "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\n\
     fn main() {\n    println!(\"{}\", add(10, 20));\n}\n",
    "generated Rust should match plan 03 §3"
  );
  assert_eq!(case.run(), "30\n");
}

/// Criteria 2 and 3: precedence is checked by arithmetic, so a wrong tree is a
/// wrong number rather than a wrong-looking tree.
#[test]
fn expressions_print_their_hand_computed_values() {
  let case = support::transpile("expressions.sdx");
  assert_eq!(
    case.run(),
    concat!(
      "14\n",   // 2 + 3 * 4, not (2 + 3) * 4 = 20
      "5\n",    // (10 - 3) - 2, left-associative
      "20\n",   // (2 + 3) * 4, parentheses override
      "true\n", // (1 + 1) == 2
      "true\n", // (1 == 1) && (2 == 2)
      "4\n",    // 1 << (1 + 1)
      "7\n",    // (-3) + 10, unary binds tighter
      "42\n",   // a typed binding
      "4\n",    // a method call
      "9\n",    // a path call
    )
  );
}

/// Criterion 4: a type round-trips verbatim, however nested.
#[test]
fn a_nested_generic_type_round_trips() {
  let rust = support::transpile_source(
    "nested_generic",
    "f = (m: Vec<HashMap<String, u8>>): usize ->\n  m.len()\n",
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
    "apply = (f: &dyn Fn(i32) -> i32, x: i32): i32 ->\n  f(x)\n",
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
