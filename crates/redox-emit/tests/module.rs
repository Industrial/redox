//! Doc 03 §13.1's `import` and doc 18 §7's `module` — plan 14.
//!
//! Doc 01 §22's three inconsistent path conventions were corrected by doc 03
//! §13.1 to `::` everywhere, *"so `.` unambiguously means field or method
//! access"*. The `::` half shipped with doc 18 §5.2; this is the half named
//! second.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// Doc 03 §13.1's five forms, each with the Rust it states.
#[test]
fn every_import_form_emits_its_use() {
  for (redox, rust) in [
    (
      "import std::collections::HashMap",
      "use std::collections::HashMap;",
    ),
    (
      "import std::io::{self, Read, Write}",
      "use std::io::{self, Read, Write};",
    ),
    (
      "import std::collections::HashMap as Map",
      "use std::collections::HashMap as Map;",
    ),
    ("import crate::models::*", "use crate::models::*;"),
    (
      "export import crate::models::User",
      "pub use crate::models::User;",
    ),
  ] {
    let out = xrs(&format!("{redox}\n"));
    assert_eq!(out, format!("{rust}\n"), "{redox}");
  }
}

/// A brace-nested tree keeps Rust's spacing — one space after each comma,
/// which is what the comma arm writes and what rustfmt leaves alone.
#[test]
fn a_nested_tree_keeps_rusts_spacing() {
  let out = xrs("import std::io::{Read, Write}\n");
  assert!(out.contains("{Read, Write}"), "{out}");
}

/// A bodiless `module` loads a sibling file.
#[test]
fn a_bodiless_module_emits_a_semicolon() {
  assert_eq!(xrs("module config\n"), "mod config;\n");
  assert_eq!(xrs("export module parser\n"), "pub mod parser;\n");
}

/// A body emits braces, and the two are different Rust — which is why
/// `ModuleDecl::items` is an `Option` and not a possibly-empty `Vec`.
#[test]
fn a_module_body_emits_braces() {
  let out = xrs("module inner\n  f : u32\n  f = 1\n");
  assert!(out.contains("mod inner {"), "{out}");
  assert!(out.contains("    fn f() -> u32 {"), "{out}");
  assert!(out.contains("        1\n"), "{out}");
}

/// Modules nest, because nothing in Rust stops them either — and this is the
/// test that the item emitter really recursed rather than special-casing one
/// level.
#[test]
fn modules_nest() {
  let out = xrs("module a\n  module b\n    f : u32\n    f = 1\n");
  assert!(out.contains("mod a {"), "{out}");
  assert!(out.contains("    mod b {"), "{out}");
  assert!(out.contains("        fn f() -> u32 {"), "{out}");
}

/// Plan 14's acceptance criterion 5: a `test` block inside a module body
/// lands in **that** module's test module. It falls out of gathering per
/// item-list rather than per file.
#[test]
fn a_test_inside_a_module_stays_inside_it() {
  let out = xrs(
    "module inner\n  test \"in here\"\n    assert 1 == 1\ntest \"out there\"\n  assert 2 == 2\n",
  );
  let inner_module = out.find("mod inner {").expect("the module");
  let inner_test = out.find("fn in_here()").expect("the inner test");
  let inner_close = out.find("\n}\n").expect("the module's close");
  assert!(inner_module < inner_test, "{out}");
  assert!(inner_test < inner_close, "the inner test is inside:\n{out}");
  assert_eq!(out.matches("mod __rdx_tests").count(), 2, "{out}");
}

/// Items keep source order, imports included — plan 14 decision 3. The blank
/// line between them is what makes it hold: rustfmt's `reorder_imports`
/// sorts within a contiguous block, so grouping them would have meant the
/// emitter replicating rustfmt's sort.
#[test]
fn items_keep_source_order() {
  let out = xrs("import z::last\nimport a::first\n");
  let z = out.find("z::last").expect("z");
  let a = out.find("a::first").expect("a");
  assert!(z < a, "source order, not sorted:\n{out}");
  assert!(
    out.contains(";\n\nuse "),
    "a blank line separates them:\n{out}"
  );
}

/// A path segment is not a type variable. `fmt::Result` has a lowercase
/// `fmt`, and `fmt` is a module — making it a generic parameter emitted
/// `fn write_ok<FMT>(…) -> FMT::Result`, which rustc rejects.
#[test]
fn a_lowercase_path_segment_is_not_a_type_variable() {
  let out = xrs("write_ok : &mut String -> fmt::Result\nwrite_ok s = s.write_str \"ok\"\n");
  assert!(
    out.contains("fn write_ok(s: &mut String) -> fmt::Result {"),
    "{out}"
  );
  assert!(!out.contains("<FMT>"), "{out}");
}

/// A trailing `*` must not continue the line: it is the last token of every
/// glob import, and the path scanner swallowed the following declaration
/// whole until the lexer stopped treating it as an operator.
#[test]
fn a_glob_import_does_not_swallow_the_next_item() {
  let out = xrs("import crate::models::*\n\nmain : ()\nmain =\n  print \"x\"\n  ()\n");
  assert!(out.contains("use crate::models::*;"), "{out}");
  assert!(out.contains("fn main() {"), "{out}");
}
