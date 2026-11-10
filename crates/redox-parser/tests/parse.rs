//! Behaviour of [`redox_parser::parse_source`], through the public API only.
//!
//! An integration test rather than a `#[cfg(test)]` module, per
//! `docs/06 - Redox Code Architecture.md` §7.

use redox_ast::{Expr, Item, Program, TriviaKind};
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
  let p = prog("main : ()\nmain =\n  println! \\-> \"hello\"\n  ()\n");
  assert_eq!(p.items.len(), 1);
  let Item::Fn(f) = &p.items[0] else {
    panic!("expected a function item")
  };
  assert_eq!(f.name.text, "main");
  // Two, not one: plan 55 §1.6 makes a `()`-returning body end in a written
  // `()`, so every `main` in this file carries a second element.
  assert_eq!(stmts(f).len(), 2);
  // A `MacroCall`, not a `Call` with an `Ident` callee — plan 64. The name
  // is a `String` on the node rather than an expression, because a macro is
  // not a value and cannot be passed as one.
  let Expr::MacroCall { name, args, .. } = &stmts(f)[0].expr else {
    panic!("expected a macro call");
  };
  assert_eq!(name, "println");
  assert_eq!(args.len(), 1);
  // The span is asserted by **slicing the source** rather than by writing the
  // offsets down, which is what the comment this replaces argued for: plan 52
  // moved every offset in this file by one when `::` became `:`, and plan 64
  // moves them all again. A slice survives both.
  let src = "main : ()\nmain =\n  println! \\-> \"hello\"\n  ()\n";
  let span = stmts(f)[0].expr.span();
  assert_eq!(&src[span.0..span.1], "println! \\-> \"hello\"");
}

/// Plan 01 acceptance criterion 6, at the AST level.
#[test]
fn nodes_carry_spans_that_index_the_source() {
  let src = "main : ()\nmain =\n  println! \\-> \"hello\"\n  ()\n";
  let p = prog(src);
  let Item::Fn(f) = &p.items[0] else {
    panic!("expected a function item")
  };
  assert_eq!(&src[f.name.span.0..f.name.span.1], "main");
  let e = &stmts(f)[0].expr;
  assert_eq!(&src[e.span().0..e.span().1], "println! \\-> \"hello\"");
}

#[test]
fn carries_a_leading_comment_onto_the_item() {
  let p = prog("-- a note\nmain : ()\nmain =\n  println! \\-> \"x\"\n  ()\n");
  let Item::Fn(f) = &p.items[0] else {
    panic!("expected a function item")
  };
  assert_eq!(f.leading.len(), 1);
  assert_eq!(f.leading[0].text, "a note");
}

#[test]
fn carries_a_trailing_comment_onto_the_statement() {
  let p = prog("main : ()\nmain =\n  println! \\-> \"x\" -- why\n  ()\n");
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
  let p =
    prog("a : ()\na =\n  println! \\-> \"1\"\n  ()\n\nb : ()\nb =\n  println! \\-> \"2\"\n  ()\n");
  assert_eq!(p.items.len(), 2);
}

#[test]
fn parses_a_file_without_a_trailing_newline() {
  assert_eq!(
    prog("main : ()\nmain =\n  println! \\-> \"x\"\n  ()")
      .items
      .len(),
    1
  );
}

#[test]
fn keeps_a_comment_at_end_of_file() {
  let p = prog("main : ()\nmain =\n  println! \\-> \"x\"\n  ()\n-- the end\n");
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
  let errs = errors("main =\n  println! \\-> \"x\"\n");
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
  let errs = errors("main : ()\nmain =\nother = ->\n  println! \\-> \"x\"\n");
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
  let errs = errors(
    "a =\n  println! \\-> \"1\"\n\nb =\n  println! \\-> \"2\"\n\nc : ()\nc =\n  println! \\-> \"3\"\n  ()\n",
  );
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
  let p = prog("main : ()\nmain =\n  total : i32 = 40 + 2\n  println! \\-> \"#{total}\"\n  ()\n");
  let Item::Fn(f) = &p.items[0] else {
    panic!("expected a function item")
  };
  let Expr::Binding(b) = &stmts(f)[0].expr else {
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
  let p = prog("main : ()\nmain =\n  y = x::u64\n  println! \\-> \"#{1}\"\n  ()\n");
  let Item::Fn(f) = &p.items[0] else {
    panic!("expected a function item")
  };
  let Expr::Binding(b) = &stmts(f)[0].expr else {
    panic!("expected a binding")
  };
  assert!(
    matches!(&b.value, Expr::Path { text, .. } if text == "x::u64"),
    "a tight `::` joins a path: {:?}",
    b.value
  );
  assert!(b.ty.is_none(), "and ascribes nothing");
}

// ---------------------------------------------------------------------------
// Plan 55 leaves 2 and 7 — the `if` is reachable in expression position, and
// there is one spelling of it.
// ---------------------------------------------------------------------------

/// An `if` parses where a value is wanted.
///
/// Before leaf 2, `prefix` reached only the one-line `then` production, so an
/// indented `if` was available as a function's tail — where `stmt` sends it to
/// `control_flow` — and was `RDX0101` anywhere a value was read.
///
/// This asserted **both** spellings until leaf 7 retired `then`. What went
/// with it is `reaches_a_then`, the depth-zero scan that chose between them,
/// and `Expr::If`'s `is_inline` bit, which recorded the choice. A test case
/// that checked the scan was not confused by `if (if a then b else c) then d
/// else e` went too: there is no scan.
#[test]
fn an_if_parses_as_a_value() {
  for src in [
    "main : ()\nmain =\n  x = if 1 > 0\n    \"b\"\n  else\n    \"s\"\n  println! \\-> \"#{x}\"\n  ()\n",
    // An `else if` rung, which is the case that proves the chain survives
    // being read as a value.
    "main : ()\nmain =\n  x = if 1 > 0\n    \"b\"\n  else if 1 > 2\n    \"m\"\n  else\n    \"s\"\n  println! \\-> \"#{x}\"\n  ()\n",
    // And nested, which is what the retired scan was tested against: the
    // inner `if` is a parenthesised argument, which plan 65 made writable
    // and which is what paid for retiring `then`.
    "main : ()\nmain =\n  x = if (if 1 > 0\n    2\n  else\n    3) > 1\n    \"b\"\n  else\n    \"s\"\n  println! \\-> \"#{x}\"\n  ()\n",
  ] {
    let p = prog(src);
    assert_eq!(p.items.len(), 1);
  }
}

/// A bare indented block is an expression, and `'name` labels one — leaf 7.
///
/// `do` and `do 'search` spelled these, and the word is now `RDX0114`. The
/// third case is the one that could not be written at all before: a block as
/// a binding's value with nothing on the `=` line.
#[test]
fn a_block_is_an_expression() {
  for src in [
    "main : ()\nmain =\n  x =\n    1\n  println! \\-> \"#{x}\"\n  ()\n",
    "main : ()\nmain =\n  x = 'search\n    1\n  println! \\-> \"#{x}\"\n  ()\n",
    "main : ()\nmain =\n  x =\n    a = 1\n    a + 1\n  println! \\-> \"#{x}\"\n  ()\n",
  ] {
    let p = prog(src);
    assert_eq!(p.items.len(), 1);
  }
}

/// A statement-position `if` is still the indented form, by arrival.
///
/// `stmt` asks `control_flow` before it reads an expression at all, so this
/// needs no lookahead and leaf 2 did not give it one. The assertion is that an
/// `else`-less `if` — which only the statement form permits — still parses.
#[test]
fn a_statement_if_needs_no_else() {
  let _ = prog("main : ()\nmain =\n  if 1 > 0\n    println! \\-> \"y\"\n  ()\n");
}
