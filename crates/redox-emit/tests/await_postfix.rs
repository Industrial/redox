//! Doc 03 §14.3's postfix `.await` — plan 15.
//!
//! A **correction**, not a feature. Doc 01 §24 specified prefix `await`,
//! which Rust does not have; §14.3's reason for the postfix form is that it
//! *"composes with method chains, with `?`, and with the receiver pipeline of
//! §5.3; prefix requires parenthesising to do any of them"*.
//!
//! Verbatim passthrough, so these tests assert the three positions that
//! composition claim names, and nothing about runtime behaviour — there is no
//! executor (doc 01 §24), which plan 15 §3 and §6 record.

use redox_emit::emit;
use redox_parser::parse_source;

fn body(src: &str) -> String {
  emit(&parse_source(&format!("main :: ()\nmain =\n  {src}\n")).expect("should parse"))
}

#[test]
fn await_is_postfix() {
  let out = body("r = (fetch id).await");
  assert!(out.contains("let r = fetch(id).await;"), "{out}");
}

/// The two postfixes compose in source order, which is the first of §14.3's
/// three claims.
#[test]
fn await_composes_with_the_try_operator() {
  let out = body("r = (fetch id).await?");
  assert!(out.contains("let r = fetch(id).await?;"), "{out}");
}

/// And in the other order — **but `?` and `.` need a space between them.**
///
/// `?.` is one token: doc 03 §12.3's optional chaining. So `x?.await` is that
/// operator followed by a reserved word, not two postfixes, and it is a parse
/// error rather than a silently different reading. Two of doc 03's own
/// constructs collide on those two characters, which plan 15 §10 records.
#[test]
fn the_try_operator_before_await_needs_a_space() {
  let out = body("r = (fetch id)? .await");
  assert!(out.contains("let r = fetch(id)?.await;"), "{out}");
}

/// The collision itself, asserted so that implementing `?.` (row 28) cannot
/// quietly change what `x?.await` means without this failing.
#[test]
fn question_dot_is_one_token_and_not_two_postfixes() {
  let failed = parse_source("main :: ()\nmain =\n  r = (fetch id)?.await\n");
  assert!(
    failed.is_err(),
    "`?.` is doc 03 §12.3's operator, not `?` then `.`"
  );
}

/// A method chain after it — §14.3's second claim, and the shape its own
/// example uses twice.
#[test]
fn await_composes_with_a_method_chain() {
  let out = body("r = (fetch id).await.json ()");
  assert!(out.contains("let r = fetch(id).await.json();"), "{out}");
}

/// An operator receiver is parenthesised, on the same rule a method call's
/// is — which is why doc 18 §2.1 names `(get u).await` among its surviving
/// required-parens cases.
#[test]
fn an_operator_receiver_keeps_its_parentheses() {
  let out = body("r = (1 + 2).await");
  assert!(out.contains("let r = (1 + 2).await;"), "{out}");
}

/// `await` is a reserved word and not a field name, so the field arm never
/// sees it — plan 15 decision 2. This is the test that the node is its own.
#[test]
fn await_is_not_a_field() {
  let out = body("r = (fetch id).await");
  assert!(!out.contains(".await()"), "not a method call:\n{out}");
  assert!(!out.contains("await: "), "not a field:\n{out}");
}

/// **`xs |> await` is a parse error**, which is stronger than plan 15's
/// criterion 5 asked for: `await` is not an expression at all, so a pipeline
/// cannot take it as an operand. The construct is `(…).await`.
#[test]
fn await_is_not_a_pipeline_operand() {
  let failed = parse_source("main :: ()\nmain =\n  r = xs |> await\n");
  assert!(failed.is_err(), "`await` is not an expression");
}
