//! Doc 18 §§2.2 and 5.3: the signature line's generic list and bound clause,
//! and §2.2's `where` locals.
//!
//! "Lowercase type names are type variables … the generic list is the order
//! of their first appearance, lifetimes included", and "the bound clause is
//! read off the `=>` constraint". Both are *synthesised*: neither is written.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

#[test]
fn a_lowercase_type_becomes_a_generic_parameter() {
  let out = xrs("id : t -> t\nid x = x\n");
  assert!(out.contains("fn id<T>(x: T) -> T {"), "{out}");
}

/// Nested occurrences are rewritten too. The first version of the
/// substitution replaced a type only when the *whole* type was the variable,
/// which left `&[t]` alone and so left a `t` rustc could not resolve.
#[test]
fn a_nested_variable_is_rewritten() {
  let out = xrs("len_of : &Vec<t> -> usize\nlen_of xs = .len xs\n");
  assert!(
    out.contains("fn len_of<T>(xs: &Vec<T>) -> usize {"),
    "{out}"
  );
}

/// Order is first appearance, which is what keeps the output byte-stable.
#[test]
fn variables_keep_their_order_of_first_appearance() {
  let out = xrs("pick : u -> t -> u\npick a _ = a\n");
  assert!(out.contains("fn pick<U, T>("), "{out}");
}

/// Lifetimes come first, which is rustc's requirement rather than a style: a
/// type parameter before a lifetime is an error, not a lint.
#[test]
fn lifetimes_come_before_type_parameters() {
  let out = xrs("tag : &'a t -> &'a t\ntag x = x\n");
  assert!(out.contains("fn tag<'a, T>(x: &'a T) -> &'a T {"), "{out}");
}

/// `'static` needs no declaration.
#[test]
fn static_is_not_a_lifetime_parameter() {
  let out = xrs("name : () -> &'static str\nname _ = \"x\"\n");
  assert!(!out.contains("<'static>"), "{out}");
}

/// Doc 18 §5.3 taken literally makes `i32` a type variable. It cannot mean
/// that, and a closed list of Rust's primitives is the correction — this is
/// the test that the correction holds.
#[test]
fn a_primitive_is_not_a_type_variable() {
  let out = xrs("add : i32 -> i32 -> i32\nadd a b = a + b\n");
  assert_eq!(
    out, "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    "{out}"
  );
}

#[test]
fn every_primitive_stays_concrete() {
  for name in ["u8", "usize", "f64", "bool", "char", "str", "i128"] {
    let out = xrs(&format!("f : &{name} -> usize\nf x = 1\n"));
    assert!(!out.contains('<'), "{name} is concrete:\n{out}");
  }
}

/// The bound clause is read off `=>`, and doc 18 §2.2's lowering puts it in a
/// `where` clause rather than inline in the generic list.
#[test]
fn a_constraint_becomes_a_where_clause() {
  let out = xrs("largest : Ord t => Option<t> -> t\nlargest xs = 1\n");
  assert!(out.contains("fn largest<T>(xs: Option<T>) -> T\n"), "{out}");
  assert!(out.contains("where\n    T: Ord,\n{"), "{out}");
}

/// A `where` clause ends a line, so the body's brace goes on the next one —
/// which is what rustfmt does and the only form it leaves alone.
#[test]
fn the_brace_follows_a_where_clause_on_its_own_line() {
  let out = xrs("f : Ord t => t -> t\nf x = x\n");
  assert!(
    !out.contains(", {"),
    "no stray brace on the bound line:\n{out}"
  );
  assert!(!out.contains(" {\n    x"), "{out}");
}

/// Haskell writes `where` locals *after* the expression that uses them and
/// Rust needs them before it, so moving them up is half of doc 03 §8.3's
/// lowering — plan 61 added the other half, the dependency sort.
///
/// **The fifth copy of that sentence**, and the one plan 61 §11.5 missed when
/// it counted four. Amended rather than deleted: the claim it makes about
/// *position* is still true and still worth asserting.
#[test]
fn where_locals_come_before_the_body() {
  let out = xrs("area : f64 -> f64\narea w = scaled\n  where\n    scaled = w * 2.0\n");
  let let_at = out.find("let scaled").expect("the local");
  let use_at = out.rfind("scaled\n").expect("the use");
  assert!(let_at < use_at, "{out}");
}

/// Several locals keep their order, because a later one may use an earlier.
#[test]
fn several_where_locals_keep_their_order() {
  let out = xrs(
    "area : f64 -> f64\narea w = scaled\n  where\n    base = w * 2.0\n    scaled = base * 3.0\n",
  );
  let base_at = out.find("let base").expect("base");
  let scaled_at = out.find("let scaled").expect("scaled");
  assert!(base_at < scaled_at, "{out}");
}

/// A clause with `where` locals is never collapsed to `pat => expr,`: the
/// locals need statements, and an arm with statements needs its braces.
#[test]
fn an_arm_with_where_locals_keeps_its_braces() {
  let out = xrs("step : u64 -> u64\nstep 0 = 0\nstep n = doubled\n  where\n    doubled = n * 2\n");
  assert!(out.contains("n => {"), "{out}");
  assert!(out.contains("let doubled = n * 2;"), "{out}");
}
