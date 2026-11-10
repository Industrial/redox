//! Doc 18 §2.3's guard ladder.
//!
//! `| cond = body` rows under a clause's pattern row, lowering to an
//! `if`/`else if`/`else` chain. The ladder replaces three constructs doc 03
//! had grown over the same ground — `cond`, `when` arm guards, and §6.2's
//! relational patterns — which is why §2.3 calls the bar the highest-yield
//! token in the redesign.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// §2.3's own example, including that `otherwise` becomes a bare `else`.
#[test]
fn a_ladder_becomes_an_if_else_chain() {
  let rust = xrs(
    "tier : u64 -> &'static str\n\
     tier spend\n  \
       | spend > 10_000 = \"platinum\"\n  \
       | spend > 1_000 = \"gold\"\n  \
       | otherwise = \"basic\"\n",
  );
  assert!(rust.contains("if spend > 10_000 {"), "{rust}");
  assert!(rust.contains("} else if spend > 1_000 {"), "{rust}");
  assert!(rust.contains("} else {"), "{rust}");
  assert!(
    !rust.contains("otherwise"),
    "otherwise is not emitted:\n{rust}"
  );
}

/// `otherwise` must not become `if true`. A conditional chain has an implicit
/// `else` of `()`, so a guarded function with a return type would stop
/// compiling — and rustc, not Redox, is what notices. Doc 18 §1.
#[test]
fn otherwise_does_not_become_a_condition() {
  let rust = xrs(
    "tier : u64 -> &'static str\n\
     tier spend\n  \
       | spend > 10 = \"high\"\n  \
       | otherwise = \"low\"\n",
  );
  assert!(!rust.contains("if true"), "{rust}");
  // The chain closes, so the whole `if` is one expression with a value.
  // Narrow enough to be the one-line form, which is what rustfmt does to a
  // two-armed `if`/`else` in a function body.
  assert!(
    rust.contains("if spend > 10 { \"high\" } else { \"low\" }"),
    "{rust}"
  );
}

/// A ladder is a tail position exactly as a plain body is: the value of the
/// chain is the function's value, with no `return` the author did not write.
#[test]
fn a_ladder_needs_no_return() {
  let rust = xrs(
    "sign : i32 -> i32\n\
     sign n\n  \
       | n > 0 = 1\n  \
       | otherwise = 0\n",
  );
  assert!(!rust.contains("return"), "{rust}");
}

/// Guards and several clauses compose: each clause carries its own ladder, and
/// the ladder lands inside that clause's `match` arm.
#[test]
fn a_ladder_lives_inside_a_match_arm() {
  let rust = xrs(
    "step : u64 -> u64\n\
     step 0 = 0\n\
     step n\n  \
       | n > 100 = 100\n  \
       | otherwise = n\n",
  );
  assert!(rust.contains("match "), "{rust}");
  assert!(rust.contains("0 => 0,"), "{rust}");
  assert!(rust.contains("if n > 100 {"), "{rust}");
}

/// A guard with no `otherwise` still emits — Redox does not decide whether a
/// ladder is total, so the chain reaches rustc and rustc reports the type.
#[test]
fn a_ladder_without_otherwise_still_emits() {
  let rust = xrs("noisy : ()\nnoisy\n  | 1 > 0 = print \"hi\"\n");
  assert!(rust.contains("if 1 > 0 {"), "{rust}");
  assert!(!rust.contains("else"), "nothing is invented:\n{rust}");
}
