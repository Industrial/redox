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

// --- §5.1 and §5.2: the one `if` form --------------------------------------

/// An `if` as a **value**, which is one spelling since plan 55 leaf 7 retired
/// `then`.
///
/// This read `tier = if n > 9 then "big" else "small"` and asserted the same
/// Rust, which is the point: the braces come from `one_line_if` asking what
/// rustfmt would do with the width, not from the author having written a
/// one-line conditional. So the retirement cost this lowering nothing.
///
/// The `if` stays on the `=` line, which is `examples/control.sdx`'s
/// `labelled`. Pushing it to the next line is **also** legal and means the
/// same thing — leaf 7 made a bare indented block an expression, so the value
/// becomes a block holding one `if` and Rust gets a redundant `{ … }` that
/// rustfmt keeps. Worth knowing rather than discovering.
#[test]
fn an_if_as_a_value_is_an_expression() {
  assert_eq!(
    body("  tier = if n > 9\n    \"big\"\n  else\n    \"small\"\n"),
    "let tier = if n > 9 { \"big\" } else { \"small\" };"
  );
}

#[test]
fn an_if_with_an_indented_block_is_a_statement() {
  assert_eq!(
    body("  if n > 0\n    println! \\-> \"yes\"\n"),
    "if n > 0 {\n    println!(\"yes\");\n}"
  );
}

/// An `else if` chain is an `else` whose branch is another `if`, which is how
/// Rust reads it too — so it nests without flattening.
#[test]
fn else_if_chains_as_rust_does() {
  assert_eq!(
    body(
      "  if n > 0\n    println! \\-> \"p\"\n  else if n < 0\n    println! \\-> \"n\"\n  else\n    println! \\-> \"z\"\n"
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
    body("  for x in xs\n    println! \\-> \"#{x}\"\n"),
    "for x in xs {\n    println!(\"{}\", x);\n}"
  );
}

/// A range needs no special case: `1 .. 4` is an ordinary expression, and the
/// receiver rule is what parenthesises it where `.` would bind tighter.
#[test]
fn for_over_a_range_needs_no_special_case() {
  assert_eq!(
    body("  for c in 1 .. 4\n    println! \\-> \"#{c}\"\n"),
    "for c in 1..4 {\n    println!(\"{}\", c);\n}"
  );
}

#[test]
fn loop_emits_loop() {
  assert_eq!(
    body("  loop\n    println! \\-> \"#{1}\"\n"),
    "loop {\n    println!(\"{}\", 1);\n}"
  );
}

// --- §5.6: the block expression -------------------------------------------

/// An indented block as a value — `v = do` with a block under it until plan
/// 55 leaf 7 retired the keyword, and the same Rust either way.
#[test]
fn a_block_is_an_expression() {
  assert_eq!(body("  v =\n    1\n"), "let v = {\n    1\n};");
}

/// `'name` on a block is a labelled block — legal Rust since 1.65, and how a
/// value leaves a loop that is not a function's last expression. The one
/// capability `do` carried that nothing else provides, so plan 55 §1.7 kept
/// it and dropped the word in front.
#[test]
fn a_labelled_block_emits_a_labelled_block() {
  assert_eq!(
    body("  v = 'search\n    1\n"),
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
  assert!(parse_source("main : ()\nmain =\n  'outer\n  println! \\-> \"#{1}\"\n").is_err());
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

/// The five compound operators, which were the only assignment spelling docs
/// 01, 03 and 18 defined, and a dependency of loops being usable — a `while`
/// whose body cannot change its condition does not terminate. Plan 20 §10
/// recorded that no row owned plain assignment; plan 54 is that row, and
/// `plain_assignment_emits_rusts_bare_equals` below is its half of this pair.
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

/// Both halves of the boundary plan 54 moved.
///
/// This test used to assert one half alone, and plan 20 §10.2 said why: it was
/// the record of a limit, kept *"so the limit is a tested fact and not an
/// accident waiting to be 'fixed'"*. Plan 54 is the row that fixed it, so the
/// test is rewritten rather than deleted — the limit it recorded is gone and
/// the distinction it protected is not.
///
/// `=` is still a **binding**. That is what makes a second token necessary:
/// telling a rebinding from a binding by looking at `x = v` needs the scope
/// tracking AGENTS.md forbids Redox growing, and `:=` means the two readings
/// never compete for one spelling.
#[test]
fn a_plain_equals_is_still_a_binding() {
  assert_eq!(body("  n = 1\n"), "let n = 1;");
}

/// And `:=` assigns, emitting Rust's bare `=`.
///
/// Redox's two spellings collapse to Rust's one, which is the shape of the
/// whole row: `let` is what tells Rust a binding from an assignment, and
/// Redox has no `let` to lean on.
#[test]
fn plain_assignment_emits_rusts_bare_equals() {
  assert_eq!(body("  n := 1\n"), "n = 1;");
}

/// Every place form, because `*slot := v` and `row[i] := v` are what a loop
/// actually needs and plan 20 §10.1 describes what omitting them costs.
///
/// The last row is Rust's destructuring assignment, stable since 1.59. It
/// falls out of the target being a general expression; plan 54 §6 declined it
/// before measuring that it was free, and §10 amends that. Nothing here
/// checks that the target *is* a place — `(1 + 2) := 3` is rustc's `E0070`,
/// and *"every check rustc can make, rustc makes"*.
#[test]
fn assignment_targets_every_place_form() {
  for (sdx, want) in [
    ("n := 1", "n = 1;"),
    ("self.value := 1", "self.value = 1;"),
    ("v[0] := 1", "v[0] = 1;"),
    ("*r := 1", "*r = 1;"),
    ("(a, b) := (b, a)", "(a, b) = (b, a);"),
  ] {
    assert_eq!(body(&format!("  {sdx}\n")), want, "{sdx}");
  }
}

/// Plan 55 leaf 2: the branches of an `if` **used as a value** are tail
/// positions.
///
/// `control_text` passed `tail: false` for one leaf, because leaf 1 wrote it
/// for a position the grammar could not yet produce. Leaf 2 produced it, and
/// the branches emitted `"big";` — the same defect leaf 0 fixed for tail
/// position, in the one path leaf 0 could not reach. rustc answered `E0308`.
#[test]
fn an_if_used_as_a_value_has_tail_branches() {
  let out = body("  x = if 1 > 0\n    \"b\"\n  else\n    \"s\"\n");
  assert!(
    !out.contains("\"b\";"),
    "the branch is the value, so it carries no semicolon:\n{out}"
  );
  assert!(out.contains("\"b\""), "{out}");
}

/// And an `else`-less `if` in statement position still is not one.
///
/// `if c {{ v }}` as a function's tail is `()` by Rust's own rule, so
/// `emit_control` forwards `tail` only when there is an `else`. This is the
/// assertion that keeps that carve-out — without it, a one-armed `if` would
/// start claiming to be its block's value.
#[test]
fn an_else_less_if_keeps_its_semicolons() {
  let out = body("  if 1 > 0\n    println! \\-> \"y\"\n");
  assert!(out.contains(");"), "{out}");
}
