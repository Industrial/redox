//! Doc 18 §2.3's `case … of`.
//!
//! Rust's `match`, with `->` arms. A spelling rather than a lowering: the
//! arms, the guards and the alternation are Rust's already, and what changes
//! is the arrow — which is the single reason `=>` is free to mean a
//! constraint (doc 18 §7), and why doc 03's `switch` keyword leaves with it.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// Doc 18 §2.3's own example, and the lowering it specifies.
#[test]
fn the_doc_example_lowers_to_a_match() {
  let out = xrs(
    "label :: u32 -> &'static str\n\
     label age = case age of\n  \
       n | n > 65 -> \"senior\"\n  \
       18..=64 -> \"adult\"\n  \
       Teen or PreTeen -> \"young\"\n  \
       _ -> \"newborn\"\n",
  );
  assert!(out.contains("match age {"), "{out}");
  assert!(out.contains("n if n > 65 => \"senior\","), "{out}");
  assert!(out.contains("18..=64 => \"adult\","), "{out}");
  assert!(out.contains("Teen | PreTeen => \"young\","), "{out}");
  assert!(out.contains("_ => \"newborn\","), "{out}");
}

/// The `|` after a pattern is **the guard**, which is unambiguous only
/// because alternation is spelled `or` (doc 18 §5.4). This is the test that
/// would fail if alternation ever took Rust's bar.
#[test]
fn the_bar_after_a_pattern_is_the_guard() {
  let out = xrs("f :: u32 -> u32\nf n = case n of\n  m | m > 0 -> m\n  _ -> 0\n");
  assert!(out.contains("m if m > 0 => m,"), "{out}");
  assert!(!out.contains("m | m"), "not alternation:\n{out}");
}

/// An exclusive range keeps its own spelling.
#[test]
fn an_exclusive_range_is_not_made_inclusive() {
  let out = xrs("f :: u32 -> u32\nf n = case n of\n  0..10 -> 1\n  _ -> 0\n");
  assert!(out.contains("0..10 => 1,"), "{out}");
}

/// `case` is an **expression**, so it is a binding's value as readily as a
/// clause body — no statement form, and no second lowering.
#[test]
fn a_case_is_an_expression() {
  let out =
    xrs("main :: ()\nmain =\n  r = case 1 of\n    1 -> \"one\"\n    _ -> \"other\"\n  print r\n");
  assert!(out.contains("let r = match 1 {"), "{out}");
  assert!(out.contains("1 => \"one\","), "{out}");
}

/// The scrutinee needs no terminator of its own: `of` is a reserved word, so
/// the Pratt loop stops at it the way it stops at any keyword. That is the
/// second thing the keyword commit bought.
#[test]
fn the_scrutinee_may_be_a_whole_expression() {
  let out = xrs("f :: u32 -> u32\nf n = case n + 1 of\n  1 -> 1\n  _ -> 0\n");
  assert!(out.contains("match n + 1 {"), "{out}");
}
