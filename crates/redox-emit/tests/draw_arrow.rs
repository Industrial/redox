//! Doc 18 §3.1's draw arrow in statement position.
//!
//! `name <- expr` lowers to a **callback whose body is every statement after
//! it** — doc 03 §7.6's `use` lowering, character for character, under the
//! token doc 18 gives it.
//!
//! The first test here is a regression test for the worst defect this project
//! has shipped: before `<-` was a token, `x <- f` lexed as `<` then `-`,
//! parsed as `x < -f`, and emitted **valid Rust that compared `x` against
//! negated `f`** with no diagnostic at all. Doc 02 §5.2 names that failure
//! mode — an expression that reads as Rust and means something else — as the
//! worst available, and it is the reason doc 18 §8 declines `/=` for
//! inequality.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// The regression. `x <- f` must not be a comparison.
#[test]
fn a_draw_arrow_is_not_a_comparison_against_a_negation() {
  let out = xrs("main : ()\nmain =\n  with x = f\n  println! \\-> \"#{x}\"\n  ()\n");
  assert!(
    !out.contains("x < -f"),
    "`x = f` must not lower to a comparison:\n{out}"
  );
  assert!(out.contains("f(|x| {"), "it is a callback bind:\n{out}");
}

/// One draw arrow wraps the statements that follow it.
#[test]
fn one_draw_arrow_wraps_what_follows() {
  let out = xrs(
    "main : ()\nmain =\n  with conn = pool.with_connection\n  println! \\-> \"#{conn}\"\n  ()\n",
  );
  assert_eq!(
    out,
    "fn main() {\n    \
     pool.with_connection(|conn| {\n        \
     println!(\"{}\", conn);\n        \
     ();\n    \
     });\n}\n",
    "{out}"
  );
}

/// Two nest, because the second is simply the first statement of the first
/// one's body — the flattening falls out of the recursion.
#[test]
fn two_draw_arrows_nest() {
  let out = xrs(
    "main : ()\nmain =\n  with conn = pool.with_connection\n  with tx = conn.transaction\n  println! \\-> \"#{tx}\"\n  ()\n",
  );
  assert_eq!(
    out,
    "fn main() {\n    \
     pool.with_connection(|conn| {\n        \
     conn.transaction(|tx| {\n            \
     println!(\"{}\", tx);\n            \
     ();\n        \
     });\n    \
     });\n}\n",
    "{out}"
  );
}

/// A draw arrow that is the last statement still closes its callback, even
/// with nothing inside it.
#[test]
fn a_trailing_draw_arrow_emits_an_empty_callback() {
  let out = xrs("main : ()\nmain =\n  with conn = pool.with_connection\n  ()\n");
  assert!(out.contains("pool.with_connection(|conn| {"), "{out}");
  assert!(out.contains("});"), "the callback must close:\n{out}");
}

/// With a declared return type the callback's value is the function's, so the
/// draw arrow's own `;` is omitted.
#[test]
fn a_draw_arrow_in_tail_position_omits_its_semicolon() {
  let out = xrs("f : u32\nf =\n  with conn = pool.with_connection\n  compute conn\n");
  assert!(out.contains("-> u32"), "{out}");
  assert!(
    out.contains("})\n}"),
    "the callback is the function's value, so no `;`:\n{out}"
  );
}

/// An ordinary binding is still a binding: `=` and `<-` are different
/// statements and must not be confused.
#[test]
fn an_ordinary_binding_is_unaffected() {
  let out = xrs("main : ()\nmain =\n  x = 1\n  println! \\-> \"#{x}\"\n  ()\n");
  assert!(out.contains("let x = 1;"), "{out}");
  assert!(!out.contains("(|x|"), "a binding is not a callback:\n{out}");
}
