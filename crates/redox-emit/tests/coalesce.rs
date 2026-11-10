//! Doc 03 §12.1's `??`, which doc 18 leaves unchanged, and doc 18 §7's unit
//! expression.
//!
//! The two lowerings that were still missing from the expression grammar.

use redox_emit::emit;
use redox_parser::parse_source;

fn body(src: &str) -> String {
  // The trailing `()` is plan 55 §1.6's unit tail, which `main` now needs.
  emit(&parse_source(&format!("main : ()\nmain =\n  {src}\n  ()\n")).expect("should parse"))
}

/// A `match`, not `unwrap_or_else`. Doc 03 §12.1's first reason is decisive
/// rather than merely persuasive: that method's closure arity differs between
/// `Option` and `Result`, so the lowering would depend on the operand's
/// *type* — and doc 03 §2's admission test 1 requires a lowering fixed by the
/// source text alone.
#[test]
fn coalesce_lowers_to_a_match() {
  let out = body("name = opt ?? \"anonymous\"");
  assert!(out.contains("match opt {"), "{out}");
  assert!(out.contains("Some(__rdx_v) => __rdx_v,"), "{out}");
  assert!(out.contains("None => \"anonymous\","), "{out}");
  assert!(!out.contains("unwrap_or_else"), "{out}");
}

/// The fallback's laziness is explicit in the generated code, which is the
/// third of §12.1's reasons: the `None` arm is only evaluated when taken.
#[test]
fn the_fallback_sits_in_the_none_arm() {
  let out = body("name = opt ?? expensive ()");
  assert!(out.contains("None => expensive(),"), "{out}");
}

/// `??` is below `||` (doc 03 §4.2's precedence), so it takes the whole
/// disjunction on its left rather than the last operand.
#[test]
fn coalesce_binds_looser_than_the_operators() {
  let out = body("r = a ?? b + 1");
  assert!(out.contains("None => b + 1,"), "{out}");
}

/// `f ()` is doc 18 §7's replacement for doc 03 §5.2's `!` zero-argument
/// suffix, so a lone unit argument **is** the empty argument list.
#[test]
fn a_lone_unit_argument_is_no_argument() {
  let out = body("r = compute ()");
  assert!(out.contains("let r = compute();"), "{out}");
  assert!(!out.contains("compute(())"), "{out}");
}

/// A unit value in its own right still emits `()`.
#[test]
fn a_unit_value_is_still_unit() {
  let out = body("r = ()");
  assert!(out.contains("let r = ();"), "{out}");
}

/// `print ()` is the blank line, for the same reason: the unit argument is
/// the empty argument list. Bare `print` is not a call at all — juxtaposition
/// needs something to juxtapose, which is the whole point of §7's `f ()`.
#[test]
fn print_unit_is_the_blank_line() {
  let out = body("print ()");
  assert!(out.contains("println!();"), "{out}");
}
