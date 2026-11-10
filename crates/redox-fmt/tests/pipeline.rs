//! Doc 03 §5.3's pipeline, at both of its widths.
//!
//! `|>` is the one binary operator with a layout of its own, because it is the
//! one the lexer continues across lines: `next_line_continues` looks ahead for
//! a line opening with `|>` and swallows it before the off-side rule can see
//! it. So a pipeline too wide for one line has exactly one multi-line spelling
//! that parses — one stage per continuation line, indented beneath the
//! receiver — and the formatter had none at all, which is how it produced an
//! 89-character line out of `examples/application.sdx`.
//!
//! The width is `PIPELINE_WIDTH`, which is rustfmt's `chain_width` and the
//! emitter's `CHAIN_WIDTH`. Both halves of the rule are asserted here, and the
//! second is the one the corpus needs: `examples/lexical.sdx` and
//! `examples/section.sdx` are full of short pipelines that must not be
//! reflowed.

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

/// A program's debug form with every span blanked — leaf 5's normaliser.
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

/// Every line that opens a pipeline continuation, with its indentation.
fn continuations(text: &str) -> Vec<&str> {
  text
    .lines()
    .filter(|line| line.trim_start().starts_with("|>"))
    .collect()
}

#[test]
fn each_width_reaches_its_canonical_form() {
  let (_, printed) = formatted("pipeline");
  against_canonical("pipeline", &printed);
}

/// The two wide pipelines break, and nothing else does.
///
/// Twelve continuation lines: four stages each for the two wide pipelines in
/// `main` and four for the clause-level one, and none for either short
/// pipeline — the one written on a line and the one the author broke anyway.
///
/// Nine until plan 57, when `numbers.iter ()` became `numbers |> .iter` and
/// each wide pipeline gained a stage. The receiver is a bare name now, so the
/// `.iter` that used to be part of it is a stage like the three after it —
/// which is the shape the row exists to produce.
#[test]
fn only_a_pipeline_too_wide_for_its_line_is_broken() {
  let (_, printed) = formatted("pipeline");
  assert_eq!(
    continuations(&printed).len(),
    12,
    "the wide pipelines should break at every stage and the short ones not at \
     all:\n{printed}"
  );
}

/// A continuation line sits one level in from the statement it continues.
///
/// Two spaces at the top level and four inside `main`, which is §3 row 11's
/// one level and the layout `examples/application.sdx` writes.
#[test]
fn a_continuation_is_one_level_in_from_its_statement() {
  let (_, printed) = formatted("pipeline");
  for line in continuations(&printed) {
    let indent = line.len() - line.trim_start().len();
    assert!(
      indent == 2 || indent == 4,
      "`{line}` is indented {indent} spaces, which is not one level in from a \
       statement at depth 0 or 1"
    );
  }
}

#[test]
fn formatting_preserves_the_program() {
  let (source, printed) = formatted("pipeline");
  let before = redox_parser::parse_source(&source).expect("the fixture should parse");
  let after = redox_parser::parse_source(&printed).expect("the output should re-parse");
  assert_eq!(
    without_spans(&before),
    without_spans(&after),
    "breaking a pipeline changed the program"
  );
}

/// The half that matters most: a broken pipeline stays broken, and a joined
/// one stays joined. A formatter that oscillates here would rewrite a file on
/// every run.
#[test]
fn formatting_is_idempotent() {
  let (_, once) = formatted("pipeline");
  let reparsed = redox_parser::parse_source(&once).expect("should re-parse");
  let twice = redox_fmt::format(&reparsed, &once);
  assert_eq!(once, twice, "formatting a pipeline is not idempotent");
}
