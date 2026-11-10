//! Doc 18 §4's sections.
//!
//! An operator with one side supplied, lowered to a one-parameter closure
//! over the reserved name `__rdx_1` — so a section is a *value* and needs no
//! special call site.

use redox_emit::emit;
use redox_parser::parse_source;

fn body(src: &str) -> String {
  emit(&parse_source(&format!("main : ()\nmain =\n  {src}\n")).expect("should parse"))
}

#[test]
fn a_right_section_supplies_the_right_operand() {
  let out = body("f = (+ 1)");
  assert!(out.contains("let f = |__rdx_1| __rdx_1 + 1;"), "{out}");
}

/// New in doc 18 §4, and what closes doc 03 §5.4's gap: with only right
/// sections, subtracting *from* a value had no section form at all.
#[test]
fn a_left_section_supplies_the_left_operand() {
  let out = body("f = (10 -)");
  assert!(out.contains("let f = |__rdx_1| 10 - __rdx_1;"), "{out}");
}

/// A comparison lowers to the **method** form, because an iterator adaptor
/// hands its closure a reference: `&i32 > 0` does not compile and
/// `(&i32).gt(&0)` does. Doc 18 §4's table and §1.1's reason.
#[test]
fn a_comparison_section_uses_the_method_form() {
  let out = body("f = (> 0)");
  assert!(out.contains("let f = |__rdx_1| __rdx_1.gt(&0);"), "{out}");
  assert!(
    !out.contains("__rdx_1 > 0"),
    "not the operator form:\n{out}"
  );
}

/// `(0 <)` and `(> 0)` are the same predicate written from either side, so
/// doc 18 §4 gives them one lowering — reached by flipping the operator
/// rather than by a second rule.
#[test]
fn a_flipped_comparison_has_the_same_lowering() {
  assert_eq!(body("f = (0 <)"), body("f = (> 0)"));
}

#[test]
fn every_comparison_has_its_method() {
  for (section, method) in [
    ("(== 0)", "eq"),
    ("(!= 0)", "ne"),
    ("(< 0)", "lt"),
    ("(> 0)", "gt"),
    ("(<= 0)", "le"),
    ("(>= 0)", "ge"),
  ] {
    let out = body(&format!("f = {section}"));
    assert!(out.contains(&format!("__rdx_1.{method}(&0)")), "{out}");
  }
}

#[test]
fn a_field_section_selects() {
  let out = body("f = (.name)");
  assert!(out.contains("let f = |__rdx_1| __rdx_1.name;"), "{out}");
}

/// A section of `|>` is a method reference **derived from the rule** rather
/// than added to it, which retires doc 03 §8.5's `&:name` sigil — and unlike
/// `&:` it tells a field from a method.
#[test]
fn a_pipeline_section_calls() {
  let out = body("f = (|> len)");
  assert!(out.contains("let f = |__rdx_1| __rdx_1.len();"), "{out}");
  assert_ne!(body("f = (.len)"), out, "a field is not a method");
}

/// `(- 1)` is negation, always. Doc 18 §4 declines the subtraction section,
/// which is what retires doc 03 decision 50's `(-1)`-versus-`(- 1)`
/// adjacency rule.
#[test]
fn a_leading_minus_is_negation_not_a_section() {
  let out = body("f = (- 1)");
  assert!(out.contains("-1"), "{out}");
  assert!(!out.contains("__rdx_1"), "not a section:\n{out}");
}

/// An ordinary grouping must still be a grouping — the section reader puts
/// the cursor back when what follows is not one.
#[test]
fn a_grouping_is_still_a_grouping() {
  let out = body("f = (1 + 2) * 3");
  assert!(out.contains("let f = (1 + 2) * 3;"), "{out}");
  assert!(!out.contains("__rdx_1"), "{out}");
}

/// A section is a value, so it is an argument with nothing added.
#[test]
fn a_section_is_an_argument() {
  let out = body("r = xs |> filter (> 0) |> count");
  assert!(
    out.contains("xs.filter(|__rdx_1| __rdx_1.gt(&0)).count()"),
    "{out}"
  );
}

/// A range is written tight, which is Rust's style and so rustfmt's — it
/// rewrites `1 .. 4` to `1..4`, and the emitter must not produce output
/// rustfmt rewrites.
#[test]
fn a_range_is_written_tight() {
  let out = body("r = (1 .. 4) |> count");
  assert!(out.contains("(1..4).count()"), "{out}");
}
