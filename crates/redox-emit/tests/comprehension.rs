//! Doc 18 §3.2's comprehension, and the list literal it shares its brackets
//! with.
//!
//! "The bar, the draw arrow and the comma do what `for` and `when` did." Doc
//! 03's second comprehension level — Dart's element-wise `if`/`for` inside a
//! literal — is deleted: Haskell has one comprehension and so does this.

use redox_emit::emit;
use redox_parser::parse_source;

fn body(src: &str) -> String {
  emit(&parse_source(&format!("main :: ()\nmain =\n  {src}\n")).expect("should parse"))
}

/// A list literal is Rust's array literal, unchanged — a spelling, not a
/// lowering.
#[test]
fn a_list_literal_is_an_array_literal() {
  let out = body("xs = [1, 2, 3]");
  assert!(out.contains("let xs = [1, 2, 3];"), "{out}");
}

#[test]
fn the_empty_list_is_the_empty_array() {
  let out = body("xs = []");
  assert!(out.contains("let xs = [];"), "{out}");
}

/// Doc 18 §6's third lowering, which has to sit **inside** the ascription
/// block rather than replace it: `C::from` alone drops the type's generic
/// arguments, so `[1, 2] :: Vec<i64>` would become a `Vec<i32>`.
#[test]
fn an_ascribed_list_becomes_a_from_call() {
  let out = body("xs = [1, 2] :: Vec<i64>");
  assert!(
    out.contains("let __rdx_t: Vec<i64> = Vec::from([1, 2]);"),
    "{out}"
  );
}

#[test]
fn one_generator_becomes_map() {
  let out = body("r = [ n * 2 | n in xs ]");
  assert!(
    out.contains("xs.into_iter().map(|n| n * 2).collect()"),
    "{out}"
  );
}

/// A guard uses `filter_map` and not `filter`, which is a deliberate
/// deviation from §3.2's shown lowering: `filter`'s closure receives a
/// *reference*, so the doc's own example writes `x != *y` — and Redox cannot
/// know when to insert that `*`, having no types.
#[test]
fn a_guard_becomes_filter_map_not_filter() {
  let out = body("r = [ n | n in xs, n > 0 ]");
  assert!(out.contains("if n > 0 { Some(n) } else { None }"), "{out}");
  assert!(!out.contains(".filter(|"), "not `filter`:\n{out}");
}

#[test]
fn several_guards_join_with_and() {
  let out = body("r = [ n | n in xs, n > 0, n < 9 ]");
  assert!(out.contains("if n > 0 && n < 9 {"), "{out}");
}

/// Two generators nest, outer first, and the inner closure takes `move`
/// because it captures the outer binder.
#[test]
fn two_generators_nest_outer_first() {
  let out = body("r = [ x * y | x in a, y in b ]");
  assert!(out.contains("a\n"), "the receiver ends its line:\n{out}");
  assert!(
    out.contains(".flat_map(move |x| b.into_iter().map(move |y| x * y))"),
    "{out}"
  );
}

/// One generator needs no `move`, because nothing outer is captured.
#[test]
fn one_generator_needs_no_move() {
  let out = body("r = [ n | n in xs ]");
  assert!(!out.contains("move"), "{out}");
}

/// A guard sits at the **innermost** level whatever its source position,
/// because every binder is in scope there — so the placement differs from
/// Haskell's in *when* a guard runs and never in what is produced.
#[test]
fn a_guard_lands_at_the_innermost_level() {
  let out = body("r = [ x | x in a, x > 0, y in b ]");
  assert!(out.contains(".filter_map(move |y|"), "{out}");
  assert!(out.contains("if x > 0 {"), "{out}");
}

/// The element type comes from the ascription, which is §3.2's own spelling.
#[test]
fn an_ascribed_comprehension_types_its_collect() {
  let out = body("r = [ c | c in cs ] :: Vec<char>");
  assert!(
    out.contains("let __rdx_t: Vec<char> = cs.into_iter()"),
    "{out}"
  );
  assert!(out.contains(".collect();"), "{out}");
}

/// The bar is **structural**, so the element must not absorb it as bitwise
/// or — doc 18 §5.4's rule, in the third place it applies.
#[test]
fn the_bar_is_not_bitwise_or() {
  let out = body("r = [ n | n in xs ]");
  assert!(!out.contains("n | n"), "{out}");
}

/// A receiver that is an operator expression needs its parentheses:
/// `1..3.into_iter()` is `1..(3.into_iter())`, which is not what the author
/// wrote and not even the same type.
#[test]
fn a_range_source_is_parenthesised() {
  let out = body("r = [ n | n in 1 .. 3 ]");
  assert!(out.contains("(1..3).into_iter()"), "{out}");
}

/// Long chains break the way rustfmt breaks them, because a comprehension's
/// chain is long by construction and plan 04's guarantee is that the emitter
/// writes nothing rustfmt would rewrite.
#[test]
fn a_long_chain_breaks_one_segment_per_line() {
  let out = body("r = [ n | n in xs, n % 2 == 1 ]");
  assert!(out.contains("xs\n"), "the receiver ends its line:\n{out}");
  assert!(out.contains("\n        .into_iter()"), "{out}");
  assert!(out.contains("\n        .collect()"), "{out}");
}

/// A short chain stays on one line, because rustfmt rejoins one that fits —
/// so always breaking would be as wrong as never breaking.
#[test]
fn a_short_chain_stays_on_one_line() {
  let out = body("r = [ n | n in xs ]");
  assert!(out.contains("xs.into_iter().map(|n| n).collect()"), "{out}");
}
