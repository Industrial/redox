//! Doc 18 §3's third and fourth positions for `<-` — a pattern test in a
//! condition and in a guard — plus doc 03 §7.1's `guard`. Plan 22.
//!
//! `<-` reads the same way everywhere, and in a condition it is doc 03 §5.5's
//! `is` with the operands in Haskell's order. Rust's `let` puts the pattern on
//! the left too, so the lowering reorders nothing.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// The body of `fn main`, dedented, so an assertion reads as the statements it
/// is about rather than as a whole file.
fn body(src: &str) -> String {
  // Plan 55 §1.6 requires a `()`-returning body to end in `()`, so the wrapper
  // writes one and the `();` it emits is dropped again — these assertions are
  // about `src`.
  let out = xrs(&format!("main : ()\nmain =\n{src}  ()\n"));
  let mut lines: Vec<&str> = out
    .lines()
    .skip(1)
    .take_while(|l| *l != "}")
    .map(|l| l.strip_prefix("    ").unwrap_or(l))
    .collect();
  assert_eq!(lines.pop(), Some("();"), "{out}");
  lines.join("\n")
}

// --- §5.1: a pattern test in an `if` --------------------------------------

#[test]
fn a_pattern_test_in_an_if_emits_if_let() {
  assert_eq!(
    body("  if Some v = x\n    print v\n"),
    "if let Some(v) = x {\n    println!(\"{}\", v);\n}"
  );
}

/// The pattern is on the **left** in both languages, so nothing is reordered —
/// which is why doc 18 §3.1 could replace `is` without touching the lowering.
#[test]
fn the_pattern_stays_on_the_left() {
  let out = body("  if Ok raw = read p\n    print raw\n");
  assert!(out.starts_with("if let Ok(raw) = read(p) {"), "{out}");
}

// --- §5.2: let-chains -----------------------------------------------------

/// **One** condition with two operands, flat, as Rust's own is. Lowered as two
/// nested `if`s it behaves identically until an `else` is attached and
/// differently from then on — doc 02 §5.2's failure mode with a delay fuse.
///
/// rustfmt breaks this chain across lines regardless of width — see
/// `condition_head` for the shape rule and the seven cases it was bisected
/// from. The chain is still **one** condition; only its layout is vertical.
#[test]
fn a_let_chain_is_one_flat_condition() {
  assert_eq!(
    body("  if Some v = x && v > 2\n    print v\n"),
    "if let Some(v) = x\n    && v > 2\n{\n    println!(\"{}\", v);\n}"
  );
}

/// The chain's `else` belongs to the whole chain, which is the thing a nested
/// lowering gets wrong.
#[test]
fn a_let_chain_takes_an_else() {
  assert_eq!(
    body("  if Some v = x && v > 2\n    print v\n  else\n    print 0\n"),
    "if let Some(v) = x\n    && v > 2\n{\n    println!(\"{}\", v);\n} else {\n    println!(\"{}\", 0);\n}"
  );
}

/// Three operands, and a pattern test that is not the first — both legal Rust,
/// so both are read.
#[test]
fn a_chain_may_hold_a_pattern_test_anywhere() {
  assert_eq!(
    body("  if n > 0 && Some v = x && v > n\n    print v\n"),
    "if n > 0\n    && let Some(v) = x\n    && v > n\n{\n    println!(\"{}\", v);\n}"
  );
}

/// The one chain shape rustfmt keeps on a single line: exactly two operands
/// with the `let` second. Asserted because it is a shape rule and not a width
/// one, so nothing else would catch its loss.
#[test]
fn a_two_operand_chain_with_the_let_last_stays_on_one_line() {
  assert_eq!(
    body("  if n > 0 && Some v = x\n    print v\n"),
    "if n > 0 && let Some(v) = x {\n    println!(\"{}\", v);\n}"
  );
}

// --- §5.3 and §5.4: `while` -----------------------------------------------

#[test]
fn a_pattern_test_in_a_while_emits_while_let() {
  assert_eq!(
    body("  while Some v = .pop stack\n    print v\n"),
    "while let Some(v) = stack.pop() {\n    println!(\"{}\", v);\n}"
  );
}

#[test]
fn a_while_let_chain_emits_as_rusts() {
  let out = body("  while Some v = q && v > 0\n    print v\n");
  assert!(
    out.starts_with("while let Some(v) = q\n    && v > 0\n{"),
    "{out}"
  );
}

// --- §5.7: `until` declines ------------------------------------------------

/// `until c` lowers to `while !(c)`, and a pattern test cannot be negated.
/// The nearest Rust — `while !matches!(x, Some(_))` — **binds nothing**, so it
/// cannot be what a binding construct meant. Plan 22 §1.3 reports instead of
/// lowering to something plausible.
#[test]
fn until_with_a_pattern_is_an_error() {
  let failure = parse_source("main : ()\nmain =\n  until Some v = x\n    print v\n")
    .expect_err("`until` cannot hold a pattern test");
  let redox_parser::ParseFailure::Parse(errors) = failure else {
    panic!("expected a parse failure, not a lex one");
  };
  assert!(
    errors
      .iter()
      .any(|e| e.code == redox_diag::Code::PatternUntil),
    "{errors:?}"
  );
}

/// An ordinary `until` is untouched — the decline is about the pattern, not
/// about the keyword.
#[test]
fn until_without_a_pattern_still_works() {
  assert_eq!(
    body("  until n > 5\n    n += 1\n"),
    "while !(n > 5) {\n    n += 1;\n}"
  );
}

// --- §5.5: `guard` ---------------------------------------------------------

/// Doc 03 §7.1's lowering, character for character, with doc 18 §3.1's
/// operator. The `;` after the closing brace is required: let-else is a `let`
/// statement.
#[test]
fn guard_emits_let_else() {
  assert_eq!(
    body("  guard Ok raw = read p else\n    return 0\n"),
    "let Ok(raw) = read(p) else {\n    return 0;\n};"
  );
}

/// Any diverging else branch, because the else body is a statement block like
/// any other and rustc is what insists it diverges.
#[test]
fn a_guards_else_takes_any_diverging_form() {
  for (sdx, want) in [
    ("return 0", "return 0;"),
    ("break", "break;"),
    ("continue", "continue;"),
    ("panic \"no\"", "panic!(\"no\");"),
  ] {
    let out = body(&format!("  guard Some v = o else\n    {sdx}\n"));
    assert_eq!(
      out,
      format!("let Some(v) = o else {{\n    {want}\n}};"),
      "{sdx}"
    );
  }
}

/// A `guard` without `else` is not Redox: let-else requires one, and a `guard`
/// that cannot exit is a `let`.
#[test]
fn a_guard_without_an_else_is_an_error() {
  assert!(parse_source("main : ()\nmain =\n  guard Some v = o\n  print v\n").is_err());
}

// --- §5.6: an arm guard ----------------------------------------------------

/// `if let` inside a match guard — `if_let_guard`, stable as of rustc 1.97.1
/// and **verified** before the leaf was written (plan 22 §1.5), because a plan
/// that promises an unstable lowering cannot ship.
#[test]
fn an_arm_guard_may_be_a_pattern_test() {
  let out = xrs("f : Option<i32> -> i32\nf o = case o of\n  y | Some v = y -> v\n  _ -> 0\n");
  assert!(out.contains("y if let Some(v) = y => v,"), "{out}");
}

/// An ordinary expression guard is untouched.
#[test]
fn an_expression_arm_guard_still_works() {
  let out = xrs("f : i32 -> i32\nf n = case n of\n  m | m > 0 -> m\n  _ -> 0\n");
  assert!(out.contains("m if m > 0 => m,"), "{out}");
}

// --- the `if` expression ---------------------------------------------------

/// A pattern test in an `if` used as a **value**.
///
/// Written `t = if Some _ = o then "yes" else "no"` until plan 55 leaf 7
/// retired `then`, and the asserted Rust is unchanged: `holds_a_pattern_test`
/// stopped its scan at `then` and now stops at the line break, which is
/// where the condition ended either way.
#[test]
fn the_if_expression_takes_a_pattern_test() {
  assert_eq!(
    body("  t = if Some _ = o\n    \"yes\"\n  else\n    \"no\"\n"),
    "let t = if let Some(_) = o { \"yes\" } else { \"no\" };"
  );
}

// --- criterion 8: the old path is unchanged -------------------------------

/// A condition with no `<-` is parsed by one `parse_expr` call and emits
/// exactly what plan 20 emitted. This is not an optimisation: parsing
/// operand-wise changes what the precedence climber sees, and `a && b || c` as
/// a chain of two operands would leave `|| c` unread.
#[test]
fn a_pattern_free_condition_is_unchanged() {
  for (sdx, want) in [
    ("if n > 0", "if n > 0 {"),
    ("if a && b", "if a && b {"),
    ("if a && b || c", "if a && b || c {"),
    ("if a || b && c", "if a || b && c {"),
    ("while a && b || c", "while a && b || c {"),
  ] {
    let out = body(&format!("  {sdx}\n    print 1\n"));
    assert!(out.starts_with(want), "{sdx} gave {out}");
  }
}

/// A `<-` inside `[ … ]` is doc 18 §3's **fourth** position — a comprehension
/// generator — so a condition holding one is an ordinary expression condition
/// and must stay one. This is why the lookahead counts brackets.
#[test]
fn a_comprehension_in_a_condition_is_not_a_pattern_test() {
  let out = body("  if .len ([ c | c in cs ]) > 0\n    print 1\n");
  assert!(out.starts_with("if "), "{out}");
  assert!(!out.contains("if let"), "{out}");
}

// --- the pattern form itself ----------------------------------------------

/// `Pattern::Apply` — the tuple-variant form. No row owned it; plan 22 §10
/// records why it landed here.
#[test]
fn a_constructor_pattern_applies_by_juxtaposition() {
  for (sdx, want) in [
    ("Some v", "Some(v)"),
    ("Ok raw", "Ok(raw)"),
    ("Some 0", "Some(0)"),
    ("Some _", "Some(_)"),
    ("Some (Some v)", "Some(Some(v))"),
    ("Event::Close d", "Event::Close(d)"),
  ] {
    let out = body(&format!("  if {sdx} = x\n    print 1\n"));
    assert!(out.starts_with(&format!("if let {want} = x {{")), "{out}");
  }
}

/// A **nullary** constructor stays a `Binding`, emitting the bare path.
/// Emitting `Close()` for a unit variant would be wrong, so the two cases stay
/// distinct — and doc 18 §2.4's `use T::*;` is what makes the bare path
/// resolve.
#[test]
fn a_nullary_constructor_emits_no_parentheses() {
  let out = body("  if Close = e\n    print 1\n");
  assert!(out.starts_with("if let Close = e {"), "{out}");
}

/// Juxtaposition in an equation's **row** is ambiguous — `f Some v` could be
/// `f (Some v)` or three patterns — so parentheses are required there, as in
/// Haskell.
#[test]
fn a_row_takes_a_parenthesised_constructor_pattern() {
  let out = xrs("depth : Option<i32> -> i32\ndepth (Some n) = n\ndepth _ = 0\n");
  assert!(out.contains("Some(n) => n,"), "{out}");
}

/// Alternation composes with application, so `Some 0 or None` alternates two
/// patterns and not four.
#[test]
fn alternation_composes_with_application() {
  let out = xrs("f : Option<i32> -> i32\nf o = case o of\n  Some 0 or None -> 0\n  _ -> 1\n");
  assert!(out.contains("Some(0) | None => 0,"), "{out}");
}
