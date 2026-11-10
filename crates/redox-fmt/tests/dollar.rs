//! Doc 18 §2.1's `$`, through the formatter.
//!
//! `$` is the one construct a formatter can **delete**. It is low-precedence
//! right-associative application whose entire purpose is to avoid a
//! parenthesis, and it lowers to nothing at all — `f $ x` and `f (x)` are the
//! same call, so no part of the compiler downstream of the parser can tell
//! them apart, and none needs to.
//!
//! The formatter is the exception, and it got this wrong: printing
//! `examples/application.sdx` turned every `$` in the file back into the
//! parenthesis `$` exists to avoid, which makes the construct unusable in any
//! codebase that runs `redox fmt`. `Expr::Call`'s `is_dollar` is the memory
//! that fixes it, and this file is what reads the bit back out.
//!
//! Three assertions, as leaf 3 and leaf 5 make them:
//!
//! 1. the **fixture pair** — every spelling in `fmt/dollar.sdx` against its
//!    canonical form in `fmt/dollar.canonical.sdx`, a file rather than a
//!    literal for AGENTS.md's reason;
//! 2. **meaning preservation** — the printed text re-parses to the program it
//!    was printed from;
//! 3. **idempotence** — printing it twice changes nothing.

use std::path::PathBuf;

fn fixtures() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fmt")
}

/// The fixture's source and its formatted form.
fn formatted(stem: &str) -> (String, String) {
  let path = fixtures().join(format!("{stem}.sdx"));
  let source =
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{} should exist", path.display()));
  let program = redox_parser::parse_source(&source)
    .unwrap_or_else(|e| panic!("{} should parse: {e:?}", path.display()));
  let printed = redox_fmt::format(&program, &source);
  (source, printed)
}

/// Compare against the blessable expectation, or rewrite it and fail.
///
/// The bless-then-fail shape is the corpus's: a changed expectation is always
/// reviewed rather than absorbed into a green build.
fn against_canonical(stem: &str, printed: &str) {
  let path = fixtures().join(format!("{stem}.canonical.sdx"));
  if std::env::var_os("REDOX_BLESS").is_some() {
    std::fs::write(&path, printed).expect("should write the expectation");
    panic!("blessed {} — review the diff", path.display());
  }
  let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| {
    panic!(
      "{} is missing; run with REDOX_BLESS=1 to create it",
      path.display()
    )
  });
  assert_eq!(
    printed,
    expected,
    "the output differs from {}",
    path.display()
  );
}

/// A program's debug form with every span blanked.
///
/// Leaf 5's normaliser, for the reason its own comment records: a
/// pretty-printed span is four lines, so filtering the lines that begin
/// `span:` leaves the numbers behind and the comparison still sees them.
///
/// `Paren` is **not** stripped here, unlike `expression_printer`'s. That is
/// the point of the file: `f $ x` and `f (x)` differ by nothing but a `Paren`
/// node and a bit, so a comparison blind to `Paren` would be blind to the
/// defect.
fn without_spans(program: &redox_ast::Program) -> String {
  let text = format!("{program:?}");
  let mut out = String::with_capacity(text.len());
  let mut rest = text.as_str();
  while let Some(at) = rest.find("span: (") {
    out.push_str(&rest[..at]);
    out.push_str("span: _");
    rest = &rest[at + "span: (".len()..];
    match rest.find(')') {
      Some(close) => rest = &rest[close + 1..],
      None => break,
    }
  }
  out.push_str(rest);
  out
}

#[test]
fn each_spelling_reaches_its_canonical_form() {
  let (_, printed) = formatted("dollar");
  against_canonical("dollar", &printed);
}

/// The defect, stated as a count: no `$` goes missing.
///
/// Counted on `" $ "` so the prose in the fixture's comments — which spells it
/// `` `$` `` — cannot pad the number, and so a `$` moved to the wrong operand
/// is still counted as present. The canonical pair above is what checks
/// *where* each one landed.
#[test]
fn no_dollar_is_deleted() {
  let (source, printed) = formatted("dollar");
  let before = source.matches(" $ ").count();
  let after = printed.matches(" $ ").count();
  assert!(
    before > 5,
    "the fixture should exercise `$` more than {before} times"
  );
  assert_eq!(
    before, after,
    "formatting changed the number of `$` operators, so one was rewritten away"
  );
}

/// A parenthesised application never *acquires* a `$`.
///
/// The two spellings have to stay distinguishable in both directions, and the
/// cheap direction to get wrong is this one: a printer that spelled every
/// one-argument call with `$` would pass the count above.
///
/// Counted on the **tree** — the fixture's own text is the fixture's, and an
/// assertion quoting a line of it would be the embedded fixture AGENTS.md
/// forbids. `Paren` is the node such a rewrite would delete and `is_dollar` is
/// the bit it would set, so counting both says it in the parser's terms.
#[test]
fn a_parenthesised_call_keeps_its_parentheses() {
  let (source, printed) = formatted("dollar");
  let before = redox_parser::parse_source(&source).expect("the fixture should parse");
  let after = redox_parser::parse_source(&printed).expect("the output should re-parse");
  let (before, after) = (format!("{before:?}"), format!("{after:?}"));
  assert_eq!(
    before.matches("Paren {").count(),
    after.matches("Paren {").count(),
    "formatting added or removed a grouping parenthesis"
  );
  assert_eq!(
    before.matches("is_dollar: true").count(),
    after.matches("is_dollar: true").count(),
    "formatting respelled a call with or without its `$`"
  );
}

#[test]
fn formatting_preserves_the_program() {
  let (source, printed) = formatted("dollar");
  let before = redox_parser::parse_source(&source).expect("the fixture should parse");
  let after = redox_parser::parse_source(&printed).expect("the formatted fixture should re-parse");
  assert_eq!(
    without_spans(&before),
    without_spans(&after),
    "formatting changed the program"
  );
}

#[test]
fn formatting_is_idempotent() {
  let (_, once) = formatted("dollar");
  let reparsed = redox_parser::parse_source(&once).expect("should re-parse");
  let twice = redox_fmt::format(&reparsed, &once);
  assert_eq!(once, twice, "formatting `$` is not idempotent");
}
