//! Doc 03 §9.6's visibility markers.
//!
//! §9.6 extends doc 01 §22, which offered only `export` → `pub` and so
//! "cannot support real library work". Doc 18 §8 declines Haskell's module
//! header for the same reason: `module M (f, g) where` cannot express Rust's
//! per-item, per-field, scoped visibility.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// Absent is private, and private has **no spelling** — which is Rust's rule
/// and Haskell's opposite, where omitting the export list makes everything
/// public.
#[test]
fn no_marker_is_private_with_no_prefix() {
  let out = xrs("f : i32 -> i32\nf n = n\n");
  assert!(out.starts_with("fn f("), "{out}");
}

#[test]
fn every_scope_has_its_spelling() {
  for (marker, want) in [
    ("export", "pub fn f("),
    ("export(crate)", "pub(crate) fn f("),
    ("export(super)", "pub(super) fn f("),
    ("export(in crate::deep)", "pub(in crate::deep) fn f("),
  ] {
    let out = xrs(&format!("{marker} f : i32 -> i32\nf n = n\n"));
    assert!(out.contains(want), "{marker}:\n{out}");
  }
}

#[test]
fn a_data_declaration_takes_a_marker() {
  let as_struct = xrs("export data P = P { x : f64 }\n");
  assert!(as_struct.contains("pub struct P {"), "{as_struct}");
  let as_enum = xrs("export(crate) data E\n  = A\n  | B\n");
  assert!(as_enum.contains("pub(crate) enum E {"), "{as_enum}");
}

/// §9.6: "Per-field and per-variant visibility use the same markers inside a
/// declaration body." This is the half doc 01 §22 could not express at all.
#[test]
fn a_record_field_takes_its_own_marker() {
  let out = xrs("export data P = P { export x : f64, y : f64 }\n");
  assert!(out.contains("pub x: f64,"), "{out}");
  assert!(out.contains("    y: f64,"), "{out}");
  assert!(!out.contains("pub y"), "{out}");
}

#[test]
fn an_alias_and_a_class_take_a_marker() {
  let alias = xrs("export(crate) type D = f64\n");
  assert!(alias.contains("pub(crate) type D = f64;"), "{alias}");
  let class = xrs("export(super) class S\n  area : &self -> f64\n");
  assert!(class.contains("pub(super) trait S {"), "{class}");
}

/// Doc 03 §12.5's example shows `pub enum`, which reads as having been
/// written `export errors`: §9.6's rule that absent means private is not
/// suspended for one construct.
#[test]
fn an_errors_declaration_obeys_the_marker() {
  let private = xrs("errors E\n  Bad String \"bad: #{0}\"\n");
  assert!(private.contains("\nenum E {"), "{private}");
  let public = xrs("export errors E\n  Bad String \"bad: #{0}\"\n");
  assert!(public.contains("pub enum E {"), "{public}");
}

/// The marker belongs to the **declaration**, so it goes on the signature
/// line. An equation carrying one is a parse error, which is the right
/// answer: there is one declaration and it has one visibility.
#[test]
fn an_equation_may_not_carry_a_marker() {
  let failed = parse_source("export f : i32 -> i32\nexport f n = n\n");
  assert!(failed.is_err(), "an equation's marker should be rejected");
}
