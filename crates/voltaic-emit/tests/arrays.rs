//! Doc 03 §10.4's array repeat — plan 29.
//!
//! The one entry in §10.4's table doc 18 did not delete. A bracketed literal
//! is an **array**; `:: Vec<T>` is how it becomes a `Vec`, which is doc 18
//! §7's ascribed-literal form replacing §10.4's published `vec![…]` lowering.
//! §10.4's own first row still said otherwise until this plan — §10.

use voltaic_emit::emit;
use voltaic_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// The body of `fn main`, dedented.
fn body(src: &str) -> String {
  // Plan 55 §1.6 requires a `()`-returning body to end in `()`, so the wrapper
  // writes one and the `();` it emits is dropped again — these assertions are
  // about `src`, and the `pop` is what keeps the unit tail asserted rather
  // than merely tolerated.
  let out = xrs(&format!("main : ()\nmain =\n{src}  ()\n"));
  let mut lines: Vec<&str> = out
    .lines()
    .skip(1)
    .take_while(|l| *l != "}")
    .map(|l| l.strip_prefix("    ").unwrap_or(l))
    .collect();
  assert_eq!(lines.pop(), Some("();"), "{out}");
  lines.join("\n")
}

/// Verbatim. Doc 03 §10.4 calls it *"pure passthrough"*.
#[test]
fn an_array_repeat_passes_through() {
  assert_eq!(body("  b = [0u8; 1024]\n"), "let b = [0u8; 1024];");
}

/// **Asserted as the thing that must not happen.** A repeat expression is a
/// fixed-size array and `vec!` is a heap allocation; both compile, so only the
/// emitted text tells them apart.
#[test]
fn an_array_repeat_is_never_a_vec() {
  let out = body("  b = [0u8; 1024]\n");
  assert!(!out.contains("vec!"), "{out}");
}

/// The `;` is the whole difference between the two bracket forms, so the list
/// path is asserted beside the repeat rather than trusted.
#[test]
fn a_list_literal_is_unchanged() {
  assert_eq!(body("  a = [1, 2, 3]\n"), "let a = [1, 2, 3];");
}

/// `:: Vec<T>` is how a literal becomes growable — doc 18 §7's ascribed form,
/// which is also why `[1, 2, 3]` is *not* a `vec!`.
#[test]
fn an_ascribed_literal_becomes_a_vec() {
  let out = body("  v = [1, 2, 3] : Vec<i32>\n");
  assert!(out.contains("Vec::from([1, 2, 3])"), "{out}");
}

/// Rust allows a `const` count, and a lowering that only handled a literal
/// would have passed a narrower test.
#[test]
fn the_count_may_be_any_expression() {
  for (sdx, want) in [
    ("[0u8; 4]", "[0u8; 4]"),
    ("[7; WIDTH]", "[7; WIDTH]"),
    ("[0; n + 1]", "[0; n + 1]"),
  ] {
    assert_eq!(
      body(&format!("  a = {sdx}\n")),
      format!("let a = {want};"),
      "{sdx}"
    );
  }
}

/// Neither nesting needs a special case. Asserted because plan 22 found a bug
/// in exactly the case it had not written down.
#[test]
fn the_two_forms_nest_either_way() {
  assert_eq!(body("  g = [[1u8; 2]; 2]\n"), "let g = [[1u8; 2]; 2];");
  assert_eq!(
    body("  l = [[0u8; 2], [1u8; 2]]\n"),
    "let l = [[0u8; 2], [1u8; 2]];"
  );
}

/// An empty list and a comprehension still take their own paths — the `;`
/// check sits after the comprehension's `|` and before the list's `,`, so
/// both neighbours are asserted.
#[test]
fn the_neighbouring_bracket_forms_still_work() {
  assert_eq!(body("  e = []\n"), "let e = [];");
  let out = body("  c = [ x | x in xs ]\n");
  assert!(out.contains("xs.into_iter()"), "{out}");
}

/// Doc 18 §7 deletes `%w` and the set literal, and plan 29 removed their
/// tokens — so neither spelling lexes. Asserted here as well as in the lexer's
/// own suite, because this is the plan that removed them.
#[test]
fn the_retired_collection_literals_do_not_parse() {
  for src in [
    "main : ()\nmain =\n  x = %w[a b]\n",
    "main : ()\nmain =\n  x = #{1, 2}\n",
  ] {
    assert!(parse_source(src).is_err(), "{src} should not parse");
  }
}
