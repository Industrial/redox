//! Doc 18 §2.4's record literal.
//!
//! One source form, `head { fields }`, and two meanings — construction and
//! update — told apart by **the case of the head**. Doc 18 §5.3 makes case
//! meaningful and §2.4 says that is "why the update form is admissible at
//! all".

use redox_emit::emit;
use redox_parser::parse_source;

fn body(src: &str) -> String {
  // The trailing `()` is plan 55 §1.6's unit tail, which `main` now needs.
  emit(&parse_source(&format!("main : ()\nmain =\n  {src}\n  ()\n")).expect("should parse"))
}

#[test]
fn an_uppercase_head_constructs() {
  let out = body("p = Point { x = 10, y = 20 }");
  assert!(out.contains("let p = Point { x: 10, y: 20 };"), "{out}");
}

/// A punned field emits Rust's own shorthand. Doc 18 §2.4's table writes
/// `Point { x: x }`, which is equivalent and which clippy's
/// `redundant_field_names` would flag in the user's build — so the doc is
/// corrected rather than followed.
#[test]
fn a_punned_field_uses_rusts_shorthand() {
  let out = body("q = Point { x, y }");
  assert!(out.contains("let q = Point { x, y };"), "{out}");
  assert!(!out.contains("x: x"), "no redundant field name:\n{out}");
}

/// A lowercase head updates, which is one of doc 18 §6's disclosed
/// lowerings: a block, because Rust's own `Point { x: 99, ..p }` needs the
/// type's *path* and the source gives only the value's name.
#[test]
fn a_lowercase_head_updates() {
  let out = body("r = p { x = 99 }");
  assert!(out.contains("let mut __rdx_r = p;"), "{out}");
  assert!(out.contains("__rdx_r.x = 99;"), "{out}");
  assert!(
    out.contains("__rdx_r\n"),
    "the block yields the record:\n{out}"
  );
  assert!(
    !out.contains(".."),
    "not Rust's struct-update syntax:\n{out}"
  );
}

/// A punned field in an *update* has no shorthand to fall back on.
#[test]
fn a_punned_update_field_assigns_the_name() {
  let out = body("r = p { x }");
  assert!(out.contains("__rdx_r.x = x;"), "{out}");
}

/// A path is judged by its last segment, so a qualified constructor still
/// constructs.
#[test]
fn a_qualified_constructor_still_constructs() {
  let out = body("p = shapes::Point { x = 1 }");
  assert!(out.contains("shapes::Point { x: 1 }"), "{out}");
  assert!(!out.contains("__rdx_r"), "{out}");
}

/// Past rustfmt's `struct_lit_width`, a literal expands — because the
/// emitter must not write a line rustfmt would re-break, nor expand one it
/// would re-join.
#[test]
fn a_wide_literal_expands() {
  let out = body("p = Point { first = 100000, second = 200000 }");
  assert!(out.contains("Point {\n"), "{out}");
  assert!(out.contains("        first: 100000,"), "{out}");
}

#[test]
fn a_narrow_literal_stays_on_one_line() {
  let out = body("p = P { a = 1, b = 2 }");
  assert!(out.contains("let p = P { a: 1, b: 2 };"), "{out}");
}

/// A record literal is an ordinary expression, so it is an argument too —
/// which is the case that would break if `{` were read as anything else.
#[test]
fn a_record_literal_is_an_argument() {
  let out = body("print (area (Point { x = 1, y = 2 }))");
  assert!(out.contains("area(Point { x: 1, y: 2 })"), "{out}");
}
