//! Doc 03 §7.6's `RDX0044` — plan 30.
//!
//! The tail of a draw arrow becomes a closure, so a `return` there leaves the
//! closure and not the enclosing function. §7.6 calls it a *"disclosed trap,
//! rejected rather than documented away"*, and the result is valid Rust that
//! means something else.
//!
//! Most of this file is about what the check must **not** reject: `?`, and a
//! `return` inside another closure. A check that caught those would break the
//! idiom the construct exists for.

use redox_diag::Code;
use redox_emit::emit;
use redox_parser::{ParseFailure, parse_source};

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// `main`'s body, with a draw arrow at the top so everything after it is a
/// tail.
fn tail(src: &str) -> String {
  format!("main :: ()\nmain =\n  conn <- pool.with_connection\n{src}")
}

fn codes(src: &str) -> Vec<Code> {
  match parse_source(src) {
    Ok(_) => Vec::new(),
    Err(ParseFailure::Lex(_)) => panic!("expected a parse failure, not a lex one"),
    Err(ParseFailure::Parse(errors)) => errors.into_iter().map(|e| e.code).collect(),
  }
}

// --- §5.1 and §5.2: what is rejected --------------------------------------

#[test]
fn a_return_in_a_tail_is_rejected() {
  assert_eq!(
    codes(&tail("  return ()\n")),
    vec![Code::ReturnInCallbackTail]
  );
}

/// Still the same closure, so still rejected — in every block a statement can
/// own.
#[test]
fn a_return_nested_in_a_tails_block_is_rejected() {
  for body in [
    "  if bad conn\n    return ()\n",
    "  while bad conn\n    return ()\n",
    "  for c in items\n    return ()\n",
    "  loop\n    return ()\n",
    "  if bad conn\n    print 1\n  else\n    return ()\n",
    "  guard Some v <- conn else\n    return ()\n",
  ] {
    assert_eq!(
      codes(&tail(body)),
      vec![Code::ReturnInCallbackTail],
      "{body}"
    );
  }
}

/// Reported **once**, which is why the check is a whole-program pass and not a
/// per-block one: `return_in_tail` recurses, so a per-block check would report
/// a nested tail's `return` twice.
#[test]
fn a_nested_return_is_reported_once() {
  let out = codes(&tail("  if bad conn\n    if worse conn\n      return ()\n"));
  assert_eq!(out, vec![Code::ReturnInCallbackTail], "reported once");
}

// --- §5.3 to §5.5: what is accepted ---------------------------------------

/// **The asymmetry.** §7.6: *"`?` is unaffected, because it propagates out of
/// the closure, which is the intended behaviour and is why the example above
/// type-checks."* A check that rejected both would break the one idiom `use`
/// exists for.
#[test]
fn a_question_mark_in_a_tail_is_accepted() {
  assert!(codes(&tail("  x = (run conn)?\n  Ok ()\n")).is_empty());
}

/// A `return` cannot appear inside a lambda **at all**: doc 18 §4's lambda
/// body is an expression and `return` is a statement, so `\x -> return x` is
/// `expected an expression`.
///
/// So plan 30 §1.4's first exclusion is **unreachable**, and this asserts the
/// reason rather than the exclusion — a test for an unreachable branch would
/// pass whatever the branch did. §10 records it.
#[test]
fn a_return_cannot_appear_inside_a_lambda() {
  assert_eq!(
    codes(&tail("  f = \\x -> return x\n  Ok ()\n")),
    vec![Code::ExpectedExpression],
    "`return` is a statement; a lambda body is an expression"
  );
}

/// What *is* reachable: a lambda in a tail with an ordinary body, which the
/// walk must not descend into looking for anything.
#[test]
fn a_lambda_in_a_tail_is_accepted() {
  assert!(codes(&tail("  f = \\x -> x + 1\n  Ok ()\n")).is_empty());
}

/// A `try` block is a closure too, so a `return` in one returning from it is
/// the construct working.
#[test]
fn a_return_inside_a_try_in_a_tail_is_accepted() {
  assert!(
    codes(&tail("  r = try\n    return 1\n  Ok ()\n")).is_empty(),
    "a `try` block is its own closure"
  );
}

/// A `return` **before** the arrow is not in the closure at all.
#[test]
fn a_return_before_the_arrow_is_accepted() {
  let src =
    "main :: ()\nmain =\n  if early\n    return ()\n  conn <- pool.with_connection\n  Ok ()\n";
  assert!(codes(src).is_empty());
}

/// And a `return` in a function with no draw arrow is untouched, which is
/// every function written before this plan.
#[test]
fn a_return_with_no_arrow_is_accepted() {
  assert!(codes("f :: i32 -> i32\nf n =\n  if n > 0\n    return n\n  0\n").is_empty());
}

// --- §5.6: a nested arrow gets its own check ------------------------------

/// Reported against the inner arrow's closure, because that is the one the
/// `return` would leave.
#[test]
fn a_nested_arrows_tail_is_checked_too() {
  let src =
    "main :: ()\nmain =\n  conn <- pool.with_connection\n  tx <- conn.transaction\n  return ()\n";
  assert_eq!(codes(src), vec![Code::ReturnInCallbackTail]);
}

// --- §5.9: the lowering is unchanged --------------------------------------

/// This plan adds a refusal and must change no lowering — plan 13's flattening
/// is asserted here as well as in its own suite, because a check that ran at
/// the wrong time could have perturbed it.
#[test]
fn the_flattening_is_unchanged() {
  let out = xrs("main :: ()\nmain =\n  conn <- pool.with_connection\n  use_it conn\n");
  assert!(out.contains("pool.with_connection(|conn| {"), "{out}");
  assert!(out.contains("use_it(conn);"), "{out}");
}
