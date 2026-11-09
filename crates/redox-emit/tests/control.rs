//! Doc 01 §8's control flow, as doc 18 §1.1 leaves it — plan 20.
//!
//! Doc 18 deletes `cond`, `unless` and every postfix form, so what remains is
//! `if`, `while`, `until`, `for`, `loop`, `do`, `break` and `continue`. Each
//! lowers to the Rust construct of the same name; `until` is the one rewrite,
//! and the parentheses it adds are the whole point of asserting it.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// The body of `fn main`, dedented, so an assertion reads as the statements it
/// is about rather than as a whole file.
fn body(src: &str) -> String {
  let out = xrs(&format!("main :: ()\nmain =\n{src}"));
  out
    .lines()
    .skip(1)
    .take_while(|l| *l != "}")
    .map(|l| l.strip_prefix("    ").unwrap_or(l))
    .collect::<Vec<_>>()
    .join("\n")
}

// --- §5.1 and §5.2: the two `if` forms ------------------------------------

/// `then` is what tells the expression from the statement — one keyword
/// decides which node the parser builds.
#[test]
fn an_if_with_then_is_an_expression() {
  assert_eq!(
    body("  tier = if n > 9 then \"big\" else \"small\"\n"),
    "let tier = if n > 9 { \"big\" } else { \"small\" };"
  );
}

#[test]
fn an_if_with_an_indented_block_is_a_statement() {
  assert_eq!(
    body("  if n > 0\n    print \"yes\"\n"),
    "if n > 0 {\n    println!(\"yes\");\n}"
  );
}

/// An `else if` chain is an `else` whose branch is another `if`, which is how
/// Rust reads it too — so it nests without flattening.
#[test]
fn else_if_chains_as_rust_does() {
  assert_eq!(
    body(
      "  if n > 0\n    print \"p\"\n  else if n < 0\n    print \"n\"\n  else\n    print \"z\"\n"
    ),
    "if n > 0 {\n    println!(\"p\");\n} else if n < 0 {\n    println!(\"n\");\n} else {\n    println!(\"z\");\n}"
  );
}

// --- §5.3: `while` and `until` --------------------------------------------

#[test]
fn while_emits_while() {
  assert_eq!(
    body("  while n < 3\n    n += 1\n"),
    "while n < 3 {\n    n += 1;\n}"
  );
}

/// **The one assertion this plan exists for.** `until a && b` is
/// `while !(a && b)`, not `while !a && b` — the two agree on the first
/// iteration and part company on the second, so nothing but the parentheses
/// distinguishes a correct lowering from a silently wrong one.
#[test]
fn until_parenthesises_its_whole_condition() {
  assert_eq!(
    body("  until n > 5 && n % 2 == 0\n    n += 1\n"),
    "while !(n > 5 && n % 2 == 0) {\n    n += 1;\n}"
  );
}

/// A single-term condition is parenthesised too. Emitting `while !n` for
/// `until n` would be correct and emitting it only there would be a special
/// case, so there is none.
#[test]
fn until_parenthesises_a_single_term_as_well() {
  assert_eq!(
    body("  until done\n    n += 1\n"),
    "while !(done) {\n    n += 1;\n}"
  );
}

// --- §5.4 and §5.5: `for` and `loop` --------------------------------------

#[test]
fn for_draws_from_its_source_with_in() {
  assert_eq!(
    body("  for x in xs\n    print x\n"),
    "for x in xs {\n    println!(\"{}\", x);\n}"
  );
}

/// A range needs no special case: `1 .. 4` is an ordinary expression, and the
/// receiver rule is what parenthesises it where `.` would bind tighter.
#[test]
fn for_over_a_range_needs_no_special_case() {
  assert_eq!(
    body("  for c in 1 .. 4\n    print c\n"),
    "for c in 1..4 {\n    println!(\"{}\", c);\n}"
  );
}

#[test]
fn loop_emits_loop() {
  assert_eq!(
    body("  loop\n    print 1\n"),
    "loop {\n    println!(\"{}\", 1);\n}"
  );
}

// --- §5.6: `do` ------------------------------------------------------------

#[test]
fn do_emits_a_block_expression() {
  assert_eq!(body("  v = do\n    1\n"), "let v = {\n    1\n};");
}

/// `do 'name` is a labelled block — legal Rust since 1.65, and how a value
/// leaves a loop that is not a function's last expression.
#[test]
fn a_labelled_do_emits_a_labelled_block() {
  assert_eq!(
    body("  v = do 'search\n    1\n"),
    "let v = 'search: {\n    1\n};"
  );
}

// --- §5.7: `break` and `continue` ------------------------------------------

/// All four `break` spellings and both `continue` spellings, because the value
/// and the label are independent and each pair is one line of emitter.
#[test]
fn break_and_continue_emit_their_rust() {
  for (sdx, want) in [
    ("break", "break;"),
    ("break v", "break v;"),
    ("break 'l", "break 'l;"),
    ("break 'l v", "break 'l v;"),
    ("continue", "continue;"),
    ("continue 'l", "continue 'l;"),
  ] {
    assert_eq!(
      body(&format!("  loop\n    {sdx}\n")),
      format!("loop {{\n    {want}\n}}"),
      "{sdx}"
    );
  }
}

// --- §5.8: labels ----------------------------------------------------------

/// A label reaches every loop keyword, and goes before it.
#[test]
fn a_label_prefixes_any_loop() {
  for (sdx, want) in [
    ("'a while c", "'a: while c {"),
    ("'a until c", "'a: while !(c) {"),
    ("'a for x in xs", "'a: for x in xs {"),
    ("'a loop", "'a: loop {"),
  ] {
    let out = body(&format!("  {sdx}\n    break 'a\n"));
    assert!(out.starts_with(want), "{sdx} gave {out}");
  }
}

/// A label with no loop after it is an error rather than a binding, because
/// `'outer` alone cannot be anything else.
#[test]
fn a_label_with_no_loop_is_an_error() {
  assert!(parse_source("main :: ()\nmain =\n  'outer\n  print 1\n").is_err());
}

// --- §5.9: `return` reaches through ---------------------------------------

/// §1.6 claims this for free — a loop body is a statement list like any
/// other, so the `return` statement that already existed works inside one.
/// Asserted rather than assumed, which is how plan 14 found its §1.1 wrong.
#[test]
fn return_works_inside_a_loop() {
  assert_eq!(
    body("  for d in 2 .. 20\n    if n % d == 0\n      return d\n"),
    "for d in 2..20 {\n    if n % d == 0 {\n        return d;\n    }\n}"
  );
}

// --- doc 01 §4.3: compound assignment -------------------------------------

/// The only assignment spelling docs 01, 03 and 18 define, and a dependency
/// of loops being usable — a `while` whose body cannot change its condition
/// does not terminate. Plan 20 §10 records that no row owned it.
#[test]
fn the_five_compound_operators_assign() {
  for (sdx, want) in [
    ("n += 1", "n += 1;"),
    ("n -= 1", "n -= 1;"),
    ("n *= 2", "n *= 2;"),
    ("n /= 2", "n /= 2;"),
    ("n %= 2", "n %= 2;"),
  ] {
    assert_eq!(body(&format!("  {sdx}\n")), want, "{sdx}");
  }
}

/// Doc 18 §2.5 assigns to `self.value`, so the target is an expression and not
/// a name.
#[test]
fn a_compound_assignment_targets_a_place_expression() {
  assert_eq!(body("  self.value += 1\n"), "self.value += 1;");
}

/// A plain `=` is still a binding. This is the line that makes plain
/// reassignment inexpressible, and it is deliberate: telling a rebinding from
/// a binding needs scope tracking AGENTS.md forbids.
#[test]
fn a_plain_equals_is_still_a_binding() {
  assert_eq!(body("  n = 1\n"), "let n = 1;");
}
