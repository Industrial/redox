//! Error recovery at two levels — plan 03 leaf 5, criteria 6 and 7.
//!
//! `docs/COMPILER.md` §1 measured this as the axis LALRPOP loses outright: it
//! reports one error and stops. Recursive descent gets the item level in about
//! fifteen lines, and plan 03 adds the statement level for the same cost.

use redox_parser::{ParseError, ParseFailure, parse_source};

fn errors(src: &str) -> Vec<ParseError> {
  match parse_source(src).expect_err("should fail") {
    ParseFailure::Parse(errs) => errs,
    other => panic!("expected parse errors, got {other:?}"),
  }
}

/// Criterion 6: three independent mistakes produce three diagnostics.
#[test]
fn three_errors_in_one_file_produce_three_diagnostics() {
  let src = "a =\n  print 1\n\nb =\n  print 2\n\nc =\n  print 3\n";
  assert_eq!(
    errors(src).len(),
    3,
    "every malformed item should be reported"
  );
}

/// Criterion 7: a bad statement does not discard its siblings.
#[test]
fn a_bad_statement_does_not_discard_its_siblings() {
  // The middle statement is malformed; the two around it are fine.
  let src = "main : ()\nmain =\n  print 1\n  = = =\n  print 3\n";
  let errs = errors(src);
  assert!(!errs.is_empty(), "the bad statement should be reported");
  assert_eq!(
    errs.len(),
    1,
    "only the bad statement should be reported, not its siblings: {errs:?}"
  );
}

/// Two bad statements in one body are both reported, which is what statement-
/// level recovery buys over item-level alone.
#[test]
fn two_bad_statements_in_one_body_are_both_reported() {
  let src = "main : ()\nmain =\n  = =\n  print 2\n  = =\n";
  assert_eq!(errors(src).len(), 2);
}

#[test]
fn a_well_formed_file_reports_nothing() {
  let src = "add : i32 -> i32 -> i32\nadd a b =\n  a + b\n\nmain : ()\nmain =\n  print (add 1 2)\n";
  assert!(parse_source(src).is_ok(), "should parse cleanly");
}

/// A stray closer inside a captured type **terminates**.
///
/// This is a regression test for a hang, not for a wrong answer, and it works
/// by returning: if `capture_type` loops again, this test never finishes and
/// the suite times out rather than reporting a diff.
///
/// `capture_type` tracked bracket depth and decremented on `>` without
/// checking that anything had opened, so a `>` with no `<` took `depth` to
/// **-1** at depth zero — and every termination condition in that loop,
/// `Newline` and `Eof` included, was guarded by `depth == 0`. Nothing was ever
/// tested again, `p.advance()` at `Eof` does not advance, and the captured
/// text grew until the kernel killed the process. Twenty-eight bytes.
///
/// Found by plan 43's fuzz sweep, and only after plan 52: the type marker lost
/// a character, so every seed in the corpus got a byte shorter and every
/// mutation of every seed became different bytes. The defect was older than
/// that — the typed-binding spelling below took a single `:` before plan 52
/// too, which is how authorship was settled.
///
/// A hang is worse than a panic here. Plan 40's language server runs this
/// parser on every keystroke, so this is a frozen editor in front of somebody
/// who has not finished typing.
#[test]
fn a_stray_closer_in_a_type_terminates() {
  for src in [
    // Through a signature.
    "add : i32 !> i32\nadd a =\n  a\n",
    // Through a typed binding — the spelling plan 52 did not change.
    "main : ()\nmain =\n  n : i32 !> i32 = 1\n  print n\n",
    // Through an ascription.
    "main : ()\nmain =\n  n = 1 : i32 !> i32\n  print n\n",
    // The original fuzz mutation: a `-` flipped to a carriage return.
    "add : i32 \r> i32\nadd a =\n  a\n",
    // And the closers that are not `>`.
    "add : i32 !] i32\nadd a =\n  a\n",
    "add : i32 !>> i32\nadd a =\n  a\n",
  ] {
    // The assertion is that this line is reached at all. What the parser
    // decides about the input is the caller's business and rustc's.
    let _ = parse_source(src);
  }
}
