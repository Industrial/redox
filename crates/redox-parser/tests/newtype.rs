//! `newtype`'s shape — one constructor over one field, plan 53 leaf 2.
//!
//! An integration test rather than a `#[cfg(test)]` module, per
//! `docs/06 - Redox Code Architecture.md` §7.
//!
//! Four shapes, because the refusal has two halves and the plan's own text
//! named only one. Doc 47's §11.4 measured all three bad ones compiling at
//! exit 0: `newtype Bad = Bad f64 f64` emitted `struct Bad(f64, f64);`,
//! `newtype Zero = Zero` emitted `struct Zero;`, and `newtype Multi = A | B`
//! emitted an **enum**. The fourth test is the legal form, which is what the
//! corpus depends on and what a refusal this blunt could easily break.

use redox_ast::Item;
use redox_diag::Code;
use redox_parser::{ParseFailure, parse_source};

fn errors(src: &str) -> Vec<redox_parser::ParseError> {
  match parse_source(src).expect_err("should fail") {
    ParseFailure::Parse(errs) => errs,
    other => panic!("expected parse errors, got {other:?}"),
  }
}

/// The span the first error blames, as the source text it covers.
///
/// Asserted by slicing rather than by writing an offset down, which is
/// `parse.rs`'s `nodes_carry_spans_that_index_the_source` argument: plan 52
/// moved every byte offset in that file by one when `::` became `:`.
fn blamed(src: &str) -> String {
  let errs = errors(src);
  let span = errs[0].span;
  assert_eq!(
    errs[0].code,
    Code::NewtypeShape,
    "the newtype-shape diagnostic, not a generic one: {errs:?}"
  );
  src[span.0..span.1].to_string()
}

/// Two fields is the case the plan and the leaf both named.
///
/// The caret lands on the **second** field type, not on the declaration: the
/// first field is the one a `newtype` is allowed, so blaming the whole line
/// would accuse it too.
#[test]
fn a_newtype_with_two_fields_is_refused() {
  let src = "newtype Bad = Bad f64 f64\n";
  assert_eq!(blamed(src), "f64");
  let errs = errors(src);
  assert!(
    errs[0].message.contains("exactly one field") && errs[0].message.contains("`data`"),
    "the message names the rule and suggests `data`: {}",
    errs[0].message
  );
  // The second `f64`, which starts at 22 — the first is at 18.
  assert_eq!(errs[0].span.0, 22, "the second field, not the first");
}

/// Zero fields, which **rustc does not catch**.
///
/// Doc 47 §11.3 measured `#[repr(transparent)] pub struct Unit;` compiling
/// clean, so §6's claim that a bad payload is *"rejected by rustc, not by
/// Redox"* holds only for the multi-field case. This check is the only thing
/// between the author and a `newtype` that wraps nothing.
#[test]
fn a_newtype_with_no_field_is_refused() {
  let src = "newtype Zero = Zero\n";
  assert_eq!(blamed(src), "Zero", "the constructor, not the declaration");
  let errs = errors(src);
  assert!(
    errs[0].message.contains("wraps none"),
    "the zero-field message: {}",
    errs[0].message
  );
}

/// Two constructors — the half doc 47 §11.4 found unenforced and the plan's
/// criterion 2 does not mention.
///
/// A `newtype` that lowers to an `enum` is as wrong as one that lowers to a
/// two-field tuple struct, and it is the worse of the two to leave alone:
/// `#[repr(transparent)]` on a two-variant enum is not a thing rustc accepts
/// either, so without this the keyword promised a representation the emitter
/// could not give it.
#[test]
fn a_newtype_with_two_constructors_is_refused() {
  let src = "newtype Multi = A | B\n";
  assert_eq!(blamed(src), "B", "the second constructor");
  let errs = errors(src);
  assert!(
    errs[0].message.contains("exactly one constructor"),
    "the constructor-count message, not the field one: {}",
    errs[0].message
  );
}

/// `newtype Lone`, with no `=` row at all, emitted `struct Lone;` too.
///
/// Not in the plan's three probes, and reachable: the declaration loop breaks
/// immediately when nothing follows the name. It shares the zero-field
/// message because it is the same wrong shape, and differs only in the span
/// — there is no constructor to point at, so the type name takes the caret.
#[test]
fn a_newtype_with_no_constructor_is_refused() {
  let src = "newtype Lone\n";
  assert_eq!(
    blamed(src),
    "Lone",
    "the type name, for want of a constructor"
  );
  let errs = errors(src);
  assert!(
    errs[0].message.contains("wraps none"),
    "the zero-field message: {}",
    errs[0].message
  );
}

/// The legal form still parses, which is the half that must not regress.
///
/// Both corpus `newtype`s are this shape — `examples/data.sdx:35`'s
/// `newtype Meters = Meters f64` and `examples/fallible.sdx:19`'s
/// `newtype Tag = Tag String`.
#[test]
fn a_newtype_with_one_field_still_parses() {
  let p = parse_source("newtype Meters = Meters f64\n").expect("should parse");
  let Item::Data(d) = &p.items[0] else {
    panic!("expected a data item")
  };
  assert!(d.is_newtype);
  assert_eq!(d.constructors.len(), 1);
}

/// A one-field **record** `newtype` is Haskell's accessor form and is legal.
///
/// Counted rather than refused: `struct Tag { raw: String }` is a perfectly
/// good `repr(transparent)` type, and refusing the brace form would narrow
/// `newtype` past what the rule says.
#[test]
fn a_one_field_record_newtype_still_parses() {
  let p = parse_source("newtype Tag = Tag { raw : String }\n").expect("should parse");
  let Item::Data(d) = &p.items[0] else {
    panic!("expected a data item")
  };
  assert!(d.is_newtype);
}

/// A two-field record is refused by the same count, blaming the second field.
#[test]
fn a_two_field_record_newtype_is_refused() {
  let src = "newtype Rec = Rec { a : f64, b : f64 }\n";
  assert_eq!(blamed(src), "b : f64", "the second field declaration");
}

/// `data` keeps every shape `newtype` has just lost.
///
/// The check is gated on `is_newtype` and this is what says so: the field
/// loop in `constructor_fields` serves both keywords and had to stay
/// unbounded for `data`.
#[test]
fn data_keeps_every_shape_newtype_refuses() {
  for src in [
    "data Bad = Bad f64 f64\n",
    "data Zero = Zero\n",
    "data Multi = A | B\n",
    "data Rec = Rec { a : f64, b : f64 }\n",
  ] {
    parse_source(src).unwrap_or_else(|e| panic!("`{src}` should still parse: {e:?}"));
  }
}
