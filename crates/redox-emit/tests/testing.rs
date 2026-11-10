//! Doc 03 §15's `test` block and §12.6's assertion and panic spellings, kept
//! by doc 18 §8.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

fn body(src: &str) -> String {
  xrs(&format!("main : ()\nmain =\n  {src}\n"))
}

#[test]
fn a_test_block_becomes_an_attributed_function() {
  let out = xrs("test \"adds two numbers\"\n  assert 1 == 1\n");
  assert!(out.contains("#[cfg(test)]\nmod __rdx_tests {"), "{out}");
  assert!(out.contains("    use super::*;"), "{out}");
  assert!(
    out.contains("    #[test]\n    fn adds_two_numbers() {"),
    "{out}"
  );
}

/// §15: one module per file, however many tests there are.
#[test]
fn several_tests_share_one_module() {
  let out = xrs("test \"one\"\n  assert 1 == 1\ntest \"two\"\n  assert 2 == 2\n");
  assert_eq!(out.matches("mod __rdx_tests").count(), 1, "{out}");
  assert!(out.contains("fn one()"), "{out}");
  assert!(out.contains("fn two()"), "{out}");
}

/// §15: the module is emitted **only** when a `test` block is present.
#[test]
fn a_file_without_tests_gets_no_module() {
  let out = xrs("main : ()\nmain =\n  print \"x\"\n");
  assert!(!out.contains("__rdx_tests"), "{out}");
}

/// The tests move to the end, which is the only item that does — §15 asks
/// for one module, and a module cannot be interleaved with what it tests.
#[test]
fn the_module_follows_the_rest_of_the_file() {
  let out = xrs("test \"t\"\n  assert 1 == 1\n\nmain : ()\nmain =\n  print \"x\"\n");
  let main_at = out.find("fn main()").expect("main");
  let module_at = out.find("mod __rdx_tests").expect("the module");
  assert!(main_at < module_at, "{out}");
}

/// A deterministic slug, which matters because the golden corpus compares
/// exact bytes.
#[test]
fn the_slug_is_deterministic() {
  let out = xrs("test \"rejects a NEGATIVE input!\"\n  assert 1 == 1\n");
  assert!(out.contains("fn rejects_a_negative_input()"), "{out}");
}

/// A description beginning with a digit needs the `_`, because a Rust
/// identifier cannot start with one.
#[test]
fn a_leading_digit_is_prefixed() {
  let out = xrs("test \"2 plus 2\"\n  assert 1 == 1\n");
  assert!(out.contains("fn _2_plus_2()"), "{out}");
}

/// §15's collision rule: two descriptions differing only in punctuation slug
/// to one name, so the second takes an index.
#[test]
fn a_slug_collision_takes_an_index() {
  let out =
    xrs("test \"halves, evenly\"\n  assert 1 == 1\ntest \"halves evenly\"\n  assert 2 == 2\n");
  assert!(out.contains("fn halves_evenly()"), "{out}");
  assert!(out.contains("fn halves_evenly_2()"), "{out}");
}

/// `==` becomes `assert_eq!`, which is not decoration: that macro prints both
/// values when they differ and `assert!` prints only the expression's text.
#[test]
fn an_equality_assertion_uses_assert_eq() {
  let out = body("assert add 1 2 == 3");
  assert!(out.contains("assert_eq!(add(1, 2), 3);"), "{out}");
}

#[test]
fn an_inequality_assertion_uses_assert_ne() {
  let out = body("assert x != y");
  assert!(out.contains("assert_ne!(x, y);"), "{out}");
}

#[test]
fn any_other_assertion_uses_assert() {
  let out = body("assert flag");
  assert!(out.contains("assert!(flag);"), "{out}");
}

/// Doc 03 §12.6's panic family. Bare names, which is why they need a lookup:
/// nothing else distinguishes `todo` from a function the author has not
/// defined — which is what `todo` is for.
#[test]
fn the_panic_family_becomes_macros() {
  for (src, want) in [
    ("r = todo", "todo!()"),
    ("r = unimplemented", "unimplemented!()"),
    ("r = unreachable", "unreachable!()"),
  ] {
    let out = body(src);
    assert!(out.contains(want), "{out}");
  }
}

/// `panic "message"` takes an argument, so it is a call and is lowered where
/// calls are.
#[test]
fn panic_with_a_message_is_a_call() {
  let out = body("panic \"boom\"");
  assert!(out.contains("panic!(\"boom\");"), "{out}");
}

/// An ordinary identifier is untouched, which is what keeps the lookup from
/// being a reserved-word list by the back door.
#[test]
fn an_ordinary_name_is_not_a_macro() {
  let out = body("r = todos");
  assert!(out.contains("let r = todos;"), "{out}");
}
