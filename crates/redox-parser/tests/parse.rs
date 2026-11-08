//! Behaviour of [`redox_parser::parse_source`], through the public API only.
//!
//! An integration test rather than a `#[cfg(test)]` module, per
//! `docs/06 - Redox Code Architecture.md` §7.

use redox_ast::{Expr, Ident, Item, Program, StmtKind, TriviaKind};
use redox_parser::{ParseFailure, parse_source};

fn prog(src: &str) -> Program {
  parse_source(src).expect("should parse")
}

fn errors(src: &str) -> Vec<redox_parser::ParseError> {
  match parse_source(src).expect_err("should fail") {
    ParseFailure::Parse(errs) => errs,
    other => panic!("expected parse errors, got {other:?}"),
  }
}

#[test]
fn parses_the_hello_slice() {
  let p = prog("main = ->\n  print \"hello\"\n");
  assert_eq!(p.items.len(), 1);
  let Item::Fn(f) = &p.items[0];
  assert_eq!(f.name.text, "main");
  assert_eq!(f.body.stmts.len(), 1);
  let StmtKind::Expr(Expr::Call { callee, args, .. }) = &f.body.stmts[0].kind else {
    panic!("expected a call");
  };
  assert_eq!(
    **callee,
    Expr::Ident(Ident {
      text: "print".into(),
      span: (12, 17)
    })
  );
  assert_eq!(args.len(), 1);
}

/// Plan 01 acceptance criterion 6, at the AST level.
#[test]
fn nodes_carry_spans_that_index_the_source() {
  let src = "main = ->\n  print \"hello\"\n";
  let p = prog(src);
  let Item::Fn(f) = &p.items[0];
  assert_eq!(&src[f.name.span.0..f.name.span.1], "main");
  let StmtKind::Expr(e) = &f.body.stmts[0].kind else {
    panic!("expected an expression statement");
  };
  assert_eq!(&src[e.span().0..e.span().1], "print \"hello\"");
}

#[test]
fn carries_a_leading_comment_onto_the_item() {
  let p = prog("# a note\nmain = ->\n  print \"x\"\n");
  let Item::Fn(f) = &p.items[0];
  assert_eq!(f.leading.len(), 1);
  assert_eq!(f.leading[0].text, "a note");
}

#[test]
fn carries_a_trailing_comment_onto_the_statement() {
  let p = prog("main = ->\n  print \"x\" # why\n");
  let Item::Fn(f) = &p.items[0];
  assert_eq!(
    f.body.stmts[0].trailing.as_ref().map(|t| t.text.as_str()),
    Some("why")
  );
}

#[test]
fn parses_several_items() {
  let p = prog("a = ->\n  print \"1\"\n\nb = ->\n  print \"2\"\n");
  assert_eq!(p.items.len(), 2);
}

#[test]
fn parses_a_file_without_a_trailing_newline() {
  assert_eq!(prog("main = ->\n  print \"x\"").items.len(), 1);
}

#[test]
fn keeps_a_comment_at_end_of_file() {
  let p = prog("main = ->\n  print \"x\"\n# the end\n");
  // Found by kind, not by position: the file's own final newline arrives as a
  // `BlankLines` trivium after the comment.
  assert_eq!(
    p.trailing
      .iter()
      .find(|t| t.kind == TriviaKind::LineComment)
      .map(|t| t.text.as_str()),
    Some("the end")
  );
}

#[test]
fn reports_a_missing_arrow() {
  let errs = errors("main =\n  print \"x\"\n");
  assert_eq!(errs.len(), 1);
  assert!(errs[0].message.contains("`->`"), "{}", errs[0].message);
}

#[test]
fn reports_an_empty_body() {
  let errs = errors("main = ->\nother = ->\n  print \"x\"\n");
  assert!(
    errs[0].message.contains("indented body"),
    "{}",
    errs[0].message
  );
}

/// Bake-off axis 3, with evidence: one pass reports both bad items. The
/// LALRPOP arm reports one and stops — `docs/COMPILER.md` §1.
#[test]
fn recovers_at_item_boundaries_and_reports_every_bad_item() {
  let errs = errors("a =\n  print \"1\"\n\nb =\n  print \"2\"\n\nc = ->\n  print \"3\"\n");
  assert_eq!(errs.len(), 2, "both malformed items should be reported");
}

#[test]
fn surfaces_a_lex_error_distinctly_from_a_parse_error() {
  let err = parse_source("main = ->\n\tprint \"x\"\n").expect_err("should fail");
  assert!(matches!(err, ParseFailure::Lex(_)), "{err:?}");
}
