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
    "label : u32 -> &'static str\n\
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
  let out = xrs("f : u32 -> u32\nf n = case n of\n  m | m > 0 -> m\n  _ -> 0\n");
  assert!(out.contains("m if m > 0 => m,"), "{out}");
  assert!(!out.contains("m | m"), "not alternation:\n{out}");
}

/// An exclusive range keeps its own spelling.
#[test]
fn an_exclusive_range_is_not_made_inclusive() {
  let out = xrs("f : u32 -> u32\nf n = case n of\n  0..10 -> 1\n  _ -> 0\n");
  assert!(out.contains("0..10 => 1,"), "{out}");
}

/// `case` is an **expression**, so it is a binding's value as readily as a
/// clause body — no statement form, and no second lowering.
#[test]
fn a_case_is_an_expression() {
  let out = xrs(
    "main : ()\nmain =\n  r = case 1 of\n    1 -> \"one\"\n    _ -> \"other\"\n  print r\n  ()\n",
  );
  assert!(out.contains("let r = match 1 {"), "{out}");
  assert!(out.contains("1 => \"one\","), "{out}");
}

/// Plan 55 leaf 3's criterion 3, first half: an arm body is an expression, so
/// an **indented block** is one and needs no marker in front of it.
///
/// The arm takes **no** trailing comma. rustfmt omits one after a block body
/// and writes one after every other body, so the emitter agrees with it
/// rather than being uniform — `generated_rust_is_rustfmt_clean` is what
/// notices the difference.
#[test]
fn an_indented_block_is_an_arm_body() {
  let out = xrs("f : u32 -> u32\nf n = case n of\n  0 ->\n    a = 1\n    a\n  _ -> n\n");
  assert!(out.contains("0 => {"), "{out}");
  assert!(out.contains("let a = 1;"), "{out}");
  assert!(out.contains("        }\n        _ => n,"), "{out}");
}

/// A block of one plain expression collapses to `pat => expr,`, because
/// rustfmt flattens a one-expression block body — the same decision
/// `emit_arm` makes for a clause, and for the same reason.
#[test]
fn a_block_of_one_expression_collapses() {
  let out = xrs("f : u32 -> u32\nf n = case n of\n  0 ->\n    1\n  _ -> n\n");
  assert!(out.contains("0 => 1,"), "{out}");
  assert!(!out.contains("0 => {"), "{out}");
}

/// A comment on that one line keeps the braces: a collapsed arm has no line
/// to put one on, and a comment must reach the `.xrs` (doc 02 §10.1).
#[test]
fn a_comment_keeps_a_one_expression_arm_braced() {
  let out = xrs("f : u32 -> u32\nf n = case n of\n  0 ->\n    -- why\n    1\n  _ -> n\n");
  assert!(out.contains("0 => {"), "{out}");
  assert!(out.contains("// why"), "{out}");
}

/// `-> do` is **accepted and deprecated** for this milestone, and it is the
/// same node as the marker-free block — plan 55 §10.4's point that
/// `Expr::Do` is Redox's only spelling of a block expression. So the two
/// spellings emit the same Rust, character for character, which is what makes
/// leaf 7's removal of the keyword a removal of a *spelling* rather than of a
/// form.
#[test]
fn the_do_marker_still_parses_and_means_the_same() {
  let marked = xrs("f : u32 -> u32\nf n = case n of\n  0 -> do\n    a = 1\n    a\n  _ -> n\n");
  let bare = xrs("f : u32 -> u32\nf n = case n of\n  0 ->\n    a = 1\n    a\n  _ -> n\n");
  assert_eq!(marked, bare);
}

/// Criterion 3's second half: a compound assignment is a legal arm body, and
/// so is plan 54's `:=`. `RDX0101` before this leaf — `parse_expr` stops at
/// the `+=`, and only `Parser::stmt` read what followed, which is why
/// `assign_tail` is now shared between the two positions.
#[test]
fn an_assignment_is_an_arm_body() {
  let out = xrs(
    "main : ()\n\
     main =\n  \
       mut n = 0\n  \
       case n of\n    \
         0 -> n += 1\n    \
         _ -> n := 2\n  \
       print n\n  \
       ()\n",
  );
  assert!(out.contains("0 => n += 1,"), "{out}");
  assert!(out.contains("_ => n = 2,"), "{out}");
}

/// The scrutinee needs no terminator of its own: `of` is a reserved word, so
/// the Pratt loop stops at it the way it stops at any keyword. That is the
/// second thing the keyword commit bought.
#[test]
fn the_scrutinee_may_be_a_whole_expression() {
  let out = xrs("f : u32 -> u32\nf n = case n + 1 of\n  1 -> 1\n  _ -> 0\n");
  assert!(out.contains("match n + 1 {"), "{out}");
}
