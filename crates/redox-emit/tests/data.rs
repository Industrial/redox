//! Doc 18 §2.4's `data`, `newtype` and `type`.
//!
//! **One constructor whose name matches the type is a `struct`; anything else
//! is an `enum`.** The rule is syntactic — decidable from the source text
//! alone, doc 03 §2's admission test 1 — which is why no name resolution is
//! needed and why §2.4 calls it "the whole requirement".

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// A record whose constructor matches the type.
#[test]
fn a_matching_record_constructor_is_a_struct() {
  let out = xrs("data Point = Point { x :: f64, y :: f64 }\n");
  assert!(out.contains("struct Point {"), "{out}");
  assert!(out.contains("    x: f64,"), "{out}");
  assert!(!out.contains("enum"), "{out}");
}

/// A struct body is always one field per line, because that is what rustfmt
/// does to it — unlike a *variant* body, which stays inline while it fits.
#[test]
fn a_struct_body_is_one_field_per_line() {
  let out = xrs("data P = P { x :: f64 }\n");
  assert!(out.contains("struct P {\n    x: f64,\n}\n"), "{out}");
}

#[test]
fn a_narrow_record_variant_stays_on_one_line() {
  let out = xrs("data E\n  = Click { x :: i32, y :: i32 }\n  | Close\n");
  assert!(out.contains("    Click { x: i32, y: i32 },"), "{out}");
}

/// Past rustfmt's `struct_variant_width`, a variant expands.
#[test]
fn a_wide_record_variant_expands() {
  let out =
    xrs("data E\n  = Wide { first :: SomeLongTypeName, second :: AnotherLongOne }\n  | Close\n");
  assert!(out.contains("    Wide {\n"), "{out}");
  assert!(out.contains("        first: SomeLongTypeName,"), "{out}");
}

#[test]
fn several_constructors_make_an_enum() {
  let out = xrs("data Event\n  = Key char\n  | Close\n");
  assert!(out.contains("enum Event {"), "{out}");
  assert!(out.contains("    Key(char),"), "{out}");
  assert!(out.contains("    Close,"), "{out}");
}

/// A constructor is in scope under its own name, as it is in Haskell — the
/// fourth new lowering, and the one that keeps a `case` arm spelled `Close ->`
/// a *match* rather than an irrefutable binding.
#[test]
fn an_enum_brings_its_constructors_into_scope() {
  let out = xrs("data Event\n  = Key char\n  | Close\n");
  assert!(out.contains("use Event::*;"), "{out}");
}

/// A struct does not, because its constructor is already its name.
#[test]
fn a_struct_brings_nothing_into_scope() {
  let out = xrs("data Point = Point { x :: f64 }\n");
  assert!(!out.contains("use "), "{out}");
}

/// A single constructor whose name *differs* from the type is an enum of one
/// variant, which is correct rather than a gap: `data Shape = Circle f64`
/// introduces `Circle`, and a struct named `Shape` would not.
#[test]
fn a_differing_constructor_is_an_enum_of_one() {
  let out = xrs("data Shape = Circle f64\n");
  assert!(out.contains("enum Shape {"), "{out}");
  assert!(out.contains("    Circle(f64),"), "{out}");
}

#[test]
fn a_unit_constructor_is_a_unit_struct() {
  let out = xrs("data Marker = Marker\n");
  assert_eq!(out, "struct Marker;\n", "{out}");
}

/// `newtype` and `data` lower identically for a single-field tuple, because
/// Rust has one such form and not two.
#[test]
fn newtype_and_data_agree_on_a_tuple_struct() {
  let from_newtype = xrs("newtype Meters = Meters f64\n");
  let from_data = xrs("data Meters = Meters f64\n");
  assert_eq!(from_newtype, "struct Meters(f64);\n");
  assert_eq!(from_newtype, from_data);
}

#[test]
fn deriving_becomes_a_derive_attribute() {
  let out = xrs("data P = P { x :: f64 }\n  deriving (Debug, Clone, PartialEq)\n");
  assert!(
    out.starts_with("#[derive(Debug, Clone, PartialEq)]\n"),
    "{out}"
  );
}

/// The `deriving` clause reads the same on the declaration's own line or on
/// an indented one — the same latitude a clause body has, and for the same
/// reason: a reader chooses the layout, not a different construct.
#[test]
fn deriving_reads_the_same_in_either_layout() {
  let indented = xrs("data P = P { x :: f64 }\n  deriving (Debug)\n");
  let inline = xrs("data P = P { x :: f64 } deriving (Debug)\n");
  assert_eq!(indented, inline);
}

/// A `type` alias is already Rust, type carried verbatim — including the
/// `->` inside `Fn(…) -> …`, which `capture_type` keeps because an `->`
/// after a `)` does not end a type.
#[test]
fn a_type_alias_is_already_rust() {
  let out = xrs("type Handler = Box<dyn Fn(Req) -> Res + Send>\n");
  assert_eq!(
    out, "type Handler = Box<dyn Fn(Req) -> Res + Send>;\n",
    "{out}"
  );
}

/// A declaration and a function in one file, which is the case that would
/// break if `data` consumed the line break the next item needs.
#[test]
fn a_declaration_does_not_swallow_the_next_item() {
  let out = xrs("newtype Meters = Meters f64\n\nmain :: ()\nmain =\n  print \"x\"\n");
  assert!(out.contains("struct Meters(f64);"), "{out}");
  assert!(out.contains("fn main() {"), "{out}");
}

/// `m.0` — a tuple struct whose field cannot be read is a declaration with
/// no use.
#[test]
fn a_tuple_index_reads_a_field() {
  let out = xrs("depth :: Meters -> f64\ndepth m = m.0\n");
  assert!(out.contains("m.0"), "{out}");
}
