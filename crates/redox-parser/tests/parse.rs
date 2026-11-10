//! Behaviour of [`redox_parser::parse_source`], through the public API only.
//!
//! An integration test rather than a `#[cfg(test)]` module, per
//! `docs/06 - Redox Code Architecture.md` §7.

use redox_ast::{Expr, Ident, Item, Program, StmtKind, TriviaKind};
use redox_diag::Code;
use redox_parser::{ParseFailure, parse_source};

fn prog(src: &str) -> Program {
  parse_source(src).expect("should parse")
}

/// A function's first clause's statements.
///
/// Doc 18 §2.3 made a clause body either a block or a guard ladder, so the
/// tests that predate guards go through this rather than repeating the
/// unwrap.
fn stmts(f: &redox_ast::FnDecl) -> &[redox_ast::Stmt] {
  &f.clauses[0]
    .body
    .plain()
    .expect("expected a plain body")
    .stmts
}

fn errors(src: &str) -> Vec<redox_parser::ParseError> {
  match parse_source(src).expect_err("should fail") {
    ParseFailure::Parse(errs) => errs,
    other => panic!("expected parse errors, got {other:?}"),
  }
}

#[test]
fn parses_the_hello_slice() {
  let p = prog("main : ()\nmain =\n  print \"hello\"\n");
  assert_eq!(p.items.len(), 1);
  let Item::Fn(f) = &p.items[0] else {
    panic!("expected a function item")
  };
  assert_eq!(f.name.text, "main");
  assert_eq!(stmts(f).len(), 1);
  let StmtKind::Expr(Expr::Call { callee, args, .. }) = &stmts(f)[0].kind else {
    panic!("expected a call");
  };
  // 19..24, not 20..25: `main : ()\n` is 10 bytes where `main :: ()\n` was
  // 11, so plan 52 moved every offset in this file's bodies by one. The third
  // such assertion in the repository, after `redox-diag`'s `line_col` and
  // `redox-driver`'s `narrow` — which is the argument for asserting a span by
  // slicing the source, as `nodes_carry_spans_that_index_the_source` below
  // does, rather than by writing the number down.
  assert_eq!(
    **callee,
    Expr::Ident(Ident {
      text: "print".into(),
      span: (19, 24)
    })
  );
  assert_eq!(args.len(), 1);
}

/// Plan 01 acceptance criterion 6, at the AST level.
#[test]
fn nodes_carry_spans_that_index_the_source() {
  let src = "main : ()\nmain =\n  print \"hello\"\n";
  let p = prog(src);
  let Item::Fn(f) = &p.items[0] else {
    panic!("expected a function item")
  };
  assert_eq!(&src[f.name.span.0..f.name.span.1], "main");
  let StmtKind::Expr(e) = &stmts(f)[0].kind else {
    panic!("expected an expression statement");
  };
  assert_eq!(&src[e.span().0..e.span().1], "print \"hello\"");
}

#[test]
fn carries_a_leading_comment_onto_the_item() {
  let p = prog("-- a note\nmain : ()\nmain =\n  print \"x\"\n");
  let Item::Fn(f) = &p.items[0] else {
    panic!("expected a function item")
  };
  assert_eq!(f.leading.len(), 1);
  assert_eq!(f.leading[0].text, "a note");
}

#[test]
fn carries_a_trailing_comment_onto_the_statement() {
  let p = prog("main : ()\nmain =\n  print \"x\" -- why\n");
  let Item::Fn(f) = &p.items[0] else {
    panic!("expected a function item")
  };
  assert_eq!(
    stmts(f)[0].trailing.as_ref().map(|t| t.text.as_str()),
    Some("why")
  );
}

#[test]
fn parses_several_items() {
  let p = prog("a : ()\na =\n  print \"1\"\n\nb : ()\nb =\n  print \"2\"\n");
  assert_eq!(p.items.len(), 2);
}

#[test]
fn parses_a_file_without_a_trailing_newline() {
  assert_eq!(prog("main : ()\nmain =\n  print \"x\"").items.len(), 1);
}

#[test]
fn keeps_a_comment_at_end_of_file() {
  let p = prog("main : ()\nmain =\n  print \"x\"\n-- the end\n");
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

/// Doc 18 §2.2 replaced doc 03's `->`-after-the-signature with the token that
/// begins one, so this is the one that can now go missing. Plan 52 made that
/// token `:` rather than `::`, which is why the assertion names a single
/// colon — and why it asserts on the message text at all: this is the test
/// that would otherwise keep passing while telling the author to type a
/// spelling the parser no longer accepts.
#[test]
fn reports_a_missing_signature_marker() {
  let errs = errors("main =\n  print \"x\"\n");
  assert_eq!(errs.len(), 1);
  assert!(errs[0].message.contains("`:`"), "{}", errs[0].message);
}

/// The two names must agree — the one thing doc 18's two-line form can get
/// wrong that doc 03's one-line form could not.
#[test]
fn reports_an_equation_that_names_a_different_function() {
  let errs = errors("add : i32 -> i32\nsubtract a =\n  a\n");
  // The count is not asserted: item recovery resumes at the next declaration
  // and finds the orphaned equation, so a second error is correct. What
  // matters is that the *first* one names both halves of the disagreement,
  // because that is the message the user reads and acts on.
  assert!(errs[0].message.contains("subtract"), "{}", errs[0].message);
  assert!(errs[0].message.contains("add"), "{}", errs[0].message);
}

/// And so must the arity.
#[test]
fn reports_an_arity_disagreement() {
  let errs = errors("add : i32 -> i32 -> i32\nadd a =\n  a\n");
  assert_eq!(errs.len(), 1);
  assert!(errs[0].message.contains("arity"), "{}", errs[0].message);
}

#[test]
fn reports_an_empty_body() {
  let errs = errors("main : ()\nmain =\nother = ->\n  print \"x\"\n");
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
  let errs = errors("a =\n  print \"1\"\n\nb =\n  print \"2\"\n\nc : ()\nc =\n  print \"3\"\n");
  assert_eq!(errs.len(), 2, "both malformed items should be reported");
}

#[test]
fn surfaces_a_lex_error_distinctly_from_a_parse_error() {
  let err = parse_source("main : ()\nmain =\n\tprint \"x\"\n").expect_err("should fail");
  assert!(matches!(err, ParseFailure::Lex(_)), "{err:?}");
}

// ---------------------------------------------------------------------------
// Plan 52 — the type colon. `:` annotates a type, `::` joins a path, and the
// whitespace rule doc 18 §5.2 used to carry is gone.
// ---------------------------------------------------------------------------

/// A signature line is not read as a typed binding, in either direction.
///
/// The two shapes are now one token apart — `f : i32 -> i32` against
/// `total : i32 = 42` — so the question is whether `looks_like_a_binding`
/// can mistake one for the other. It cannot: `reaches_a_binding_eq` returns
/// false at the first depth-zero `Newline`, and a signature carries no
/// depth-zero `=` before its line break, because an `impl Trait<Assoc = T>`
/// style `=` sits inside `<>` and those increment the depth.
///
/// It is moot in practice as well, which is worth knowing before relying on
/// it: a signature line is only ever read by `Parser::item`, never at
/// statement level where `looks_like_a_binding` runs. Both directions are
/// asserted anyway, because "moot in practice" is a property of today's call
/// graph rather than of the grammar.
#[test]
fn a_signature_line_is_not_read_as_a_typed_binding() {
  let p = prog("f : i32 -> i32\nf n =\n  n\n");
  assert_eq!(p.items.len(), 1, "one item, not a binding and a function");
  let Item::Fn(f) = &p.items[0] else {
    panic!("expected a function item")
  };
  assert_eq!(f.sig.params.len(), 1, "the arrow spine is the signature's");
}

/// And a typed binding still parses as one, keeping its type.
#[test]
fn a_typed_binding_still_parses_as_a_binding() {
  let p = prog("main : ()\nmain =\n  total : i32 = 40 + 2\n  print total\n");
  let Item::Fn(f) = &p.items[0] else {
    panic!("expected a function item")
  };
  let StmtKind::Binding(b) = &stmts(f)[0].kind else {
    panic!("expected a binding")
  };
  assert_eq!(
    b.ty.as_ref().map(|t| t.text.as_str()),
    Some("i32"),
    "the binding keeps the type the `:` introduced"
  );
}

/// A spaced `::` after a **literal** is a parse error.
///
/// This half was already true the moment ascription moved to `:`, because a
/// literal cannot begin a path and so there is nothing for `::` to join.
#[test]
fn a_spaced_colon_colon_after_a_literal_is_an_error() {
  let errs = errors("main : ()\nmain =\n  n = 1 :: u64\n");
  assert!(!errs.is_empty(), "`1 :: u64` should not parse");
}

/// A spaced `::` after a **name** is refused, not consumed as a path.
///
/// This is the half that was not free, and it is plan 52 criterion 2. `u64`
/// is a valid path segment, so taking every `::` unconditionally consumed
/// `x :: u64` as `x::u64` and left rustc to report an unresolved path against
/// a line nobody wrote — the exact failure mode doc 18 §5.2's whitespace rule
/// existed to prevent. Retiring that rule had to come with a diagnostic
/// rather than with silence.
///
/// This test's predecessor asserted the misparse, deliberately, so the gap
/// was visible in the suite and not only in the plan. It is now inverted:
/// same program, opposite expectation.
#[test]
fn a_spaced_colon_colon_after_a_name_is_refused() {
  let errs = errors("main : ()\nmain =\n  y = x :: u64\n");
  assert!(!errs.is_empty(), "`x :: u64` should not parse");
  assert_eq!(
    errs[0].code,
    Code::SpacedPathSeparator,
    "the spaced-separator diagnostic, not a generic one: {errs:?}"
  );
}

/// A tight `::` is still a path, which is the half that must not regress.
#[test]
fn a_tight_colon_colon_is_still_a_path() {
  let p = prog("main : ()\nmain =\n  y = x::u64\n  print 1\n");
  let Item::Fn(f) = &p.items[0] else {
    panic!("expected a function item")
  };
  let StmtKind::Binding(b) = &stmts(f)[0].kind else {
    panic!("expected a binding")
  };
  assert!(
    matches!(&b.value, Expr::Path { text, .. } if text == "x::u64"),
    "a tight `::` joins a path: {:?}",
    b.value
  );
  assert!(b.ty.is_none(), "and ascribes nothing");
}
