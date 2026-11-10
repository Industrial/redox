//! Doc 03 §6.4 and §6.5's pattern forms — plan 33.
//!
//! All of them *"pass through to native Rust syntax"* (§6.4), so the work is
//! in the parser and the risk is in telling the forms apart: a `(` is a group
//! or a tuple by its comma, and `..` means four things by position.

use redox_diag::Code;
use redox_emit::emit;
use redox_parser::{ParseFailure, parse_source};

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// The body of `fn main`, dedented.
fn body(src: &str) -> String {
  xrs(&format!("main : ()\nmain =\n{src}"))
    .lines()
    .skip(1)
    .take_while(|l| *l != "}")
    .map(|l| l.strip_prefix("    ").unwrap_or(l))
    .collect::<Vec<_>>()
    .join("\n")
}

/// The first `case` arm's text, for the forms that only make sense in one.
fn arm(pattern: &str) -> String {
  let out = xrs(&format!(
    "f : T -> i32\nf v = case v of\n  {pattern} -> 1\n  _ -> 0\n"
  ));
  out
    .lines()
    .find(|l| l.trim_end().ends_with("=> 1,"))
    .unwrap_or_else(|| panic!("no arm in {out}"))
    .trim()
    .to_string()
}

fn codes(src: &str) -> Vec<Code> {
  match parse_source(src) {
    Ok(_) => Vec::new(),
    Err(ParseFailure::Lex(_)) => panic!("expected a parse failure, not a lex one"),
    Err(ParseFailure::Parse(errors)) => errors.into_iter().map(|e| e.code).collect(),
  }
}

// --- §5.1: tuples ---------------------------------------------------------

#[test]
fn a_tuple_pattern_destructures_a_binding() {
  assert_eq!(body("  (a, b) = p\n"), "let (a, b) = p;");
}

/// Nested, because each form is independent and a lowering that worked only at
/// the top level would pass every flat test.
#[test]
fn tuple_patterns_nest() {
  assert_eq!(body("  (a, (b, c)) = p\n"), "let (a, (b, c)) = p;");
}

/// **The comma is the whole difference** between a group and a tuple. Both
/// spellings asserted together, because the distinction is the implementation.
#[test]
fn a_group_is_not_a_tuple() {
  assert_eq!(body("  (a) = p\n"), "let a = p;");
  assert_eq!(body("  (a,) = p\n"), "let (a,) = p;");
}

#[test]
fn the_unit_pattern_is_a_tuple_of_nothing() {
  assert_eq!(body("  () = f ()\n"), "let () = f();");
}

// --- §5.2 and §5.3: slices and rests -------------------------------------

#[test]
fn a_slice_pattern_destructures() {
  assert_eq!(body("  [a, b] = xs\n"), "let [a, b] = xs;");
}

/// `..` to discard and `rest @ ..` to bind — Rust's own tokens, because doc 18
/// §7 removed `...` and both of these were already Redox's. Plan 33 §1.3.
#[test]
fn a_rest_pattern_takes_rusts_spelling() {
  assert_eq!(body("  [a, .., z] = xs\n"), "let [a, .., z] = xs;");
  assert_eq!(body("  [a, rest @ ..] = xs\n"), "let [a, rest @ ..] = xs;");
}

/// Doc 03 §6.5 published `...rest`, and an author who writes it followed a
/// published specification — so the message names the correction.
#[test]
fn the_published_ellipsis_spelling_is_declined() {
  assert_eq!(
    codes("main : ()\nmain =\n  [a, ...rest] = xs\n  print a\n"),
    vec![Code::EllipsisRestPattern]
  );
}

/// **§1.4's positional rule.** The same token is a range between two patterns
/// and a rest alone, and both are asserted because nothing else would catch
/// one swallowing the other.
#[test]
fn dotdot_is_a_range_between_patterns_and_a_rest_alone() {
  assert_eq!(arm("1..10"), "1..10 => 1,");
  assert_eq!(arm("[a, ..]"), "[a, ..] => 1,");
}

// --- §5.4 and §5.5: records ----------------------------------------------

/// `=` becomes `:`, which is the one character this form changes — doc 18
/// §2.4 made the literal `=` and §5.2 left `:` no job inside braces.
#[test]
fn a_record_pattern_takes_equals_and_emits_a_colon() {
  assert_eq!(arm("Point { x = 0, y }"), "Point { x: 0, y } => 1,");
}

/// A pun needs no spelling: it is Rust's own shorthand.
#[test]
fn a_record_pattern_puns() {
  assert_eq!(arm("Point { x, y }"), "Point { x, y } => 1,");
}

#[test]
fn a_record_pattern_takes_a_rest() {
  assert_eq!(arm("Point { x, .. }"), "Point { x, .. } => 1,");
}

/// A `::`-joined path, for an enum variant. Tight only, for doc 18 §5.2's
/// reason — a spaced `::` ascribes.
#[test]
fn a_record_pattern_takes_a_path() {
  assert_eq!(arm("Event::Click { x }"), "Event::Click { x } => 1,");
}

/// Nested patterns in a field, because the vocabulary is shared.
#[test]
fn a_record_field_holds_any_pattern() {
  assert_eq!(
    arm("Point { x = Some v, y = 0 }"),
    "Point { x: Some(v), y: 0 } => 1,"
  );
}

// --- §5.6: the as-pattern -------------------------------------------------

/// `@`'s one remaining meaning after doc 18 §4.3 dropped `@name` and `@@name`.
#[test]
fn an_as_pattern_binds_the_whole() {
  assert_eq!(arm("whole @ Move x y"), "whole @ Move(x, y) => 1,");
}

#[test]
fn an_as_pattern_works_over_a_record() {
  assert_eq!(arm("whole @ Point { x }"), "whole @ Point { x } => 1,");
}

// --- §5.7: every position -------------------------------------------------

/// The vocabulary is shared, and sharing it is the point — so each form is
/// asserted in an equation's **row** and in a `<-` condition as well as in a
/// `case` arm.
#[test]
fn the_forms_work_in_an_equations_row() {
  let out = xrs("first : (i32, i32) -> i32\nfirst (a, _) = a\n");
  assert!(out.contains("(a, _) =>"), "{out}");
}

#[test]
fn the_forms_work_in_a_condition() {
  let out = body("  if Point { x = 0, .. } = p\n    print 1\n");
  assert!(out.starts_with("if let Point { x: 0, .. } = p {"), "{out}");
}

// --- §5.8: criterion 8, the old path ------------------------------------

/// A binding's left side changing type must not change what a plain name
/// emits — this is the path every existing golden takes.
#[test]
fn a_plain_binding_is_unchanged() {
  assert_eq!(body("  x = 1\n"), "let x = 1;");
  assert_eq!(body("  mut x = 1\n"), "let mut x = 1;");
  assert_eq!(body("  x: u8 = 1\n"), "let x: u8 = 1;");
}

/// And a record **literal** is still a literal: its `=` is inside the braces,
/// so the binding lookahead does not see a top-level one and `p { x = 1 }`
/// stays an update expression.
#[test]
fn a_record_literal_is_not_read_as_a_binding() {
  let out = body("  r = p { x = 1 }\n");
  assert!(out.starts_with("let r = {"), "{out}");
  assert!(out.contains("__rdx_r.x = 1;"), "{out}");
}

// --- a tuple expression ---------------------------------------------------

/// No row owned a tuple **expression**: doc 03 §6.5 assumed a tuple value and
/// §10.4's literal table lists no tuple. Without it `(a, b) = p` can only be
/// reached through a Rust function that already returns one. Plan 33 §10.
#[test]
fn a_tuple_expression_passes_through() {
  assert_eq!(body("  p = (3, 4)\n"), "let p = (3, 4);");
  assert_eq!(body("  p = (1, (2, 3))\n"), "let p = (1, (2, 3));");
}

/// And a parenthesised group is still a group, by the same comma rule the
/// pattern side uses.
#[test]
fn a_group_expression_is_not_a_tuple() {
  assert_eq!(body("  n = (1 + 2) * 3\n"), "let n = (1 + 2) * 3;");
}
