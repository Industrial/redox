//! Declarations in a body position — plan 55 leaf 5.
//!
//! Rust lets a `fn` body hold items, and doc 03 §8.3 presumed it. Redox could
//! not express one: there was no AST carrier, `Parser::stmt` never called
//! `item`, and a declaration keyword in a body reached the expression parser
//! and became `RDX0102`. Plan 55 §1.4 said Rust's acceptance made the
//! construct reachable; it made it *lowerable*, which is not the same claim.
//!
//! `examples/local.sdx` is the running witness, compiled and executed with
//! zero rustc output. These are the unit assertions underneath it.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// A function body holding `src`, with `fn main() {` and `}` stripped.
fn body(src: &str) -> String {
  let out = xrs(&format!("main : ()\nmain =\n{src}  ()\n"));
  out
    .lines()
    .skip(1)
    .take_while(|l| *l != "}")
    .map(|l| l.strip_prefix("    ").unwrap_or(l))
    .collect::<Vec<_>>()
    .join("\n")
}

/// Every declaration keyword a block may hold lowers to the nested Rust item.
///
/// One dispatch serves both positions — `emit_item`, extracted from
/// `emit_items` by this leaf — so a local declaration and a module-scope one
/// cannot drift. That matters more than it sounds: the two are the same parse,
/// so a reader who found them differing would reasonably assume the
/// difference was deliberate.
#[test]
fn every_local_declaration_lowers_to_a_nested_item() {
  for (sdx, want) in [
    ("  import std::cmp::max\n", "use std::cmp::max;"),
    ("  const K : i32 = 2\n", "const K: i32 = 2;"),
    ("  type N = i32\n", "type N = i32;"),
    ("  data M = M f64\n", "struct M(f64);"),
    ("  newtype W = W f64\n", "#[repr(transparent)]"),
    ("  errors E\n    Gone  \"gone\"\n", "enum E {"),
    ("  class C\n    f : &self -> u32\n", "trait C {"),
  ] {
    let out = body(sdx);
    assert!(
      out.contains(want),
      "`{}` should emit `{want}`, got:\n{out}",
      sdx.trim()
    );
  }
}

/// A local declaration keeps its comments, and keeps them **once**.
///
/// `peek_leading` peeks rather than consumes, so `Parser::stmt` and
/// `Parser::item` both captured the same trivia and every comment in a local
/// declaration came out twice. The `Stmt` gives its copy up, because the
/// emitters for a declaration have always printed an item's leading comments
/// — a module-scope one has no caller to do it.
///
/// Worth a test of its own because of how it failed: duplicated comments are
/// still *present*, so nothing that checks for loss notices, and
/// `generated_rust_is_rustfmt_clean` was what caught it.
#[test]
fn a_local_declarations_comment_is_emitted_once() {
  let out = body("  -- a note\n  const K : i32 = 2\n");
  assert_eq!(out.matches("a note").count(), 1, "{out}");
}
