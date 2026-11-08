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
  let src = "main = ->\n  print 1\n  = = =\n  print 3\n";
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
  let src = "main = ->\n  = =\n  print 2\n  = =\n";
  assert_eq!(errors(src).len(), 2);
}

#[test]
fn a_well_formed_file_reports_nothing() {
  let src = "add = (a: i32, b: i32): i32 ->\n  a + b\n\nmain = ->\n  print add(1, 2)\n";
  assert!(parse_source(src).is_ok(), "should parse cleanly");
}
