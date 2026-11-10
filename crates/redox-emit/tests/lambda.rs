//! Doc 18 §4's lambda and §2.1's `$`.
//!
//! A lambda lowers to a Rust closure and `$` lowers to **nothing at all** —
//! it is grouping that runs to the end of the expression, which is why a
//! nested application rarely needs brackets.

use redox_emit::emit;
use redox_parser::parse_source;

fn body(src: &str) -> String {
  emit(&parse_source(&format!("main : ()\nmain =\n  {src}\n")).expect("should parse"))
}

#[test]
fn a_lambda_is_a_closure() {
  assert!(
    body("f = \\n -> n * n").contains("let f = |n| n * n;"),
    "{}",
    body("f = \\n -> n * n")
  );
}

/// The binder's type uses the same `:` a signature line and an
/// ascription use — one notation for a type, not three.
#[test]
fn a_typed_binder_keeps_its_type() {
  let out = body("f = \\x : i32 -> x + 1");
  assert!(out.contains("let f = |x: i32| x + 1;"), "{out}");
}

#[test]
fn move_is_carried_through() {
  let out = body("f = move \\x -> x + captured");
  assert!(out.contains("let f = move |x| x + captured;"), "{out}");
}

/// Doc 18 §8 declines currying, so `\x y -> e` is **one** closure of two
/// arguments rather than nested single-parameter closures. That matches how
/// application works in §2.1, and it is the whole reason the two decisions
/// have to agree.
#[test]
fn two_binders_make_one_closure_not_two() {
  let out = body("f = \\x y -> x + y");
  assert!(out.contains("let f = |x, y| x + y;"), "{out}");
  assert!(!out.contains("|x| |y|"), "not curried:\n{out}");
}

#[test]
fn a_lambda_can_take_no_binders() {
  assert!(
    body("f = \\ -> 0").contains("let f = || 0;"),
    "{}",
    body("f = \\ -> 0")
  );
}

/// `$` is grouping, so this is a call and not an operator in the output.
#[test]
fn dollar_lowers_to_nothing() {
  let out = body("print $ f 3");
  assert!(out.contains("println!(\"{}\", f(3));"), "{out}");
  assert!(!out.contains('$'), "`$` must not survive:\n{out}");
}

/// `$` runs to the end of the expression, which is what makes it useful:
/// `f $ g x y` is `f(g(x, y))`, not `f(g)(x, y)`.
#[test]
fn dollar_extends_to_the_end_of_the_expression() {
  let out = body("r = f $ g x y");
  assert!(out.contains("let r = f(g(x, y));"), "{out}");
}

/// Ascription is looser than `$`, matching Haskell's `infixr 0` for `$`
/// against an even looser `::`.
#[test]
fn ascription_binds_looser_than_dollar() {
  let out = body("r = f $ x : u64");
  assert!(
    out.contains("let __rdx_t: u64 = f(x);"),
    "the whole application is ascribed:\n{out}"
  );
}

/// A lambda as an argument needs its parentheses — doc 18 §2.1's surviving
/// nested-application case — and reads the way the pipeline examples do.
#[test]
fn a_lambda_works_as_an_argument() {
  let out = body("r = map (\\n -> n * n) xs");
  assert!(out.contains("map(|n| n * n, xs)"), "{out}");
}
