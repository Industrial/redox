//! Behaviour of [`redox_emit::emit`], through the public API only.
//!
//! An integration test rather than a `#[cfg(test)]` module, per
//! `docs/06 - Redox Code Architecture.md` §7.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// Plan 01 §3's concrete proof, at the unit level.
#[test]
fn emits_the_hello_slice() {
  assert_eq!(
    xrs("-- The smallest Redox program.\nmain : ()\nmain =\n  print \"hello\"\n"),
    "// The smallest Redox program.\nfn main() {\n    println!(\"hello\");\n}\n"
  );
}

/// Plan 01 acceptance criterion 5.
#[test]
fn a_comment_survives_into_the_generated_rust() {
  let out = xrs("-- kept\nmain : ()\nmain =\n  print \"x\"\n");
  assert!(out.starts_with("// kept\n"), "{out}");
}

#[test]
fn an_inline_comment_survives_into_the_generated_rust() {
  let out = xrs("main : ()\nmain =\n  print \"x\" -- why\n");
  assert!(out.contains("println!(\"x\"); // why\n"), "{out}");
}

#[test]
fn a_comment_at_end_of_file_survives() {
  let out = xrs("main : ()\nmain =\n  print \"x\"\n-- the end\n");
  assert!(out.ends_with("// the end\n"), "{out}");
}

/// Plan 01 acceptance criterion 7: emission is a pure function of the tree.
#[test]
fn emission_is_deterministic() {
  let src = "-- a\nmain : ()\nmain =\n  print \"x\"\n\nother : ()\nother =\n  print \"y\"\n";
  assert_eq!(xrs(src), xrs(src));
}

#[test]
fn separates_items_with_one_blank_line() {
  assert_eq!(
    xrs("a : ()\na =\n  print \"1\"\nb : ()\nb =\n  print \"2\"\n"),
    "fn a() {\n    println!(\"1\");\n}\n\nfn b() {\n    println!(\"2\");\n}\n"
  );
}

#[test]
fn emits_four_space_indentation() {
  let out = xrs("main : ()\nmain =\n  print \"x\"\n");
  assert!(out.contains("\n    println!"), "{out}");
}

#[test]
fn a_non_print_call_keeps_its_name() {
  assert!(xrs("main : ()\nmain =\n  log \"x\"\n").contains("log(\"x\");"));
}

/// Regression: the fixture's own final newline used to append a stray blank
/// line to every generated file. Found by the end-to-end test comparing
/// against plan 01 §3's stated output, not by inspection.
#[test]
fn ends_with_exactly_one_newline() {
  for src in [
    "main : ()\nmain =\n  print \"x\"\n",
    "main : ()\nmain =\n  print \"x\"",
    "main : ()\nmain =\n  print \"x\"\n\n\n",
    "main : ()\nmain =\n  print \"x\"\n-- tail\n",
  ] {
    let out = xrs(src);
    assert!(out.ends_with('\n'), "{out:?}");
    assert!(!out.ends_with("\n\n"), "{out:?}");
  }
}

/// Line-preserving discipline: source lines 1-3 map to output lines 1-3.
#[test]
fn preserves_line_positions_for_the_slice() {
  let out = xrs("-- one\nmain : ()\nmain =\n  print \"three\"\n");
  let lines: Vec<&str> = out.lines().collect();
  assert_eq!(lines[0], "// one");
  assert_eq!(lines[1], "fn main() {");
  assert_eq!(lines[2], "    println!(\"three\");");
}
