//! Doc 03 §13.2's attributes — plan 18.
//!
//! *"Any `#[...]` or `#![...]` line is copied through verbatim, so every
//! present and future Rust attribute is reachable with no per-attribute work
//! and interoperability becomes unconditional."*
//!
//! They are **trivia**, which is plan 18 §1.3's design rather than an
//! implementation detail: an attribute is not an item, trivia is Redox's
//! mechanism for text that belongs to a construct without being one, and
//! joining it reaches every item kind and every statement at once.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

#[test]
fn an_attribute_above_a_declaration_stays_above_it() {
  let out = xrs("#[derive(Debug)]\ndata P = P { x :: f64 }\n");
  assert!(out.starts_with("#[derive(Debug)]\nstruct P {"), "{out}");
}

#[test]
fn an_attribute_above_a_function_stays_above_it() {
  let out = xrs("#[inline]\nsquare :: u32 -> u32\nsquare n = n * n\n");
  assert!(out.starts_with("#[inline]\nfn square("), "{out}");
}

/// An inner attribute keeps its `!`, and is emitted **in place** — plan 18
/// §1.4 declined gathering them to the top, because that is a reordering and
/// plan 14 decision 3 declined those without cause.
#[test]
fn an_inner_attribute_keeps_its_bang() {
  let out = xrs("#![no_std]\nmain :: ()\nmain =\n  print \"x\"\n");
  assert!(out.starts_with("#![no_std]\n"), "{out}");
}

/// Verbatim means verbatim: the text between the brackets is the author's,
/// including strings, `=` and nested brackets.
#[test]
fn every_attribute_form_round_trips() {
  for attr in [
    "cfg(test)",
    "tokio::main",
    "serde(rename = \"id\")",
    "allow(clippy::too_many_arguments)",
    "must_use = \"look at it\"",
    "cfg_attr(feature = \"x\", derive(Debug))",
  ] {
    let out = xrs(&format!("#[{attr}]\nf :: u32\nf = 1\n"));
    assert!(out.contains(&format!("#[{attr}]")), "{attr}:\n{out}");
  }
}

/// A doc comment, an attribute and the item in that order — the first thing
/// §1.3 says trivia buys, and what a separate attribute list would have lost.
#[test]
fn a_doc_comment_and_an_attribute_keep_their_order() {
  let out = xrs("-- | What it does.\n#[inline]\nf :: u32\nf = 1\n");
  let doc = out.find("/// What it does.").expect("the doc comment");
  let attr = out.find("#[inline]").expect("the attribute");
  let item = out.find("fn f()").expect("the function");
  assert!(doc < attr && attr < item, "{out}");
}

/// And the other order, because nothing reorders them.
#[test]
fn an_attribute_and_a_doc_comment_keep_their_order() {
  let out = xrs("#[inline]\n-- | What it does.\nf :: u32\nf = 1\n");
  let attr = out.find("#[inline]").expect("the attribute");
  let doc = out.find("/// What it does.").expect("the doc comment");
  assert!(attr < doc, "{out}");
}

/// A statement carries `leading` too, so this needed nothing of its own.
#[test]
fn an_attribute_above_a_statement_stays_above_it() {
  let out = xrs("main :: ()\nmain =\n  #[allow(unused)]\n  x = 1\n  print x\n");
  assert!(
    out.contains("    #[allow(unused)]\n    let x = 1;"),
    "{out}"
  );
}

/// A class member's trivia is written by `emit_member`, which already
/// indented it — so this reached a fifth position for free.
#[test]
fn an_attribute_above_a_class_member_is_indented_with_it() {
  let out = xrs("class C t\n  #[must_use]\n  f :: &t -> u32\n");
  assert!(
    out.contains("    #[must_use]\n    fn f(&self) -> u32;"),
    "{out}"
  );
}

/// §13.2: the `deriving` clause is *"a native spelling for the dominant case,
/// not a replacement"* — so both reach the same Rust.
#[test]
fn deriving_and_a_derive_attribute_agree() {
  let clause = xrs("data P = P { x :: f64 }\n  deriving (Debug)\n");
  let attribute = xrs("#[derive(Debug)]\ndata P = P { x :: f64 }\n");
  assert_eq!(clause, attribute);
}

/// An unterminated attribute is the lexer's, with the code plan 05 already
/// catalogued — this plan added no diagnostic of its own.
#[test]
fn an_unterminated_attribute_is_a_lexer_error() {
  let failed = parse_source("#[derive(Debug)\nf :: u32\nf = 1\n");
  assert!(failed.is_err());
}
