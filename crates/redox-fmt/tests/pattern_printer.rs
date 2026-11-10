//! Plan 34 leaf 4, first half: `pattern_text` prints one pattern in canonical
//! Redox.
//!
//! The same three assertions leaf 3's printer carries, for the same reasons —
//! a fixture pair for the readable contract, idempotence, and a corpus
//! round-trip for the property that would actually catch a misprint.
//!
//! # Why a pattern round-trip needs a `case` arm
//!
//! A pattern reaches the parser in four positions: a binding, a condition, a
//! `case` arm and a function clause's head. Only the arm accepts every form —
//! a binding's pattern must be irrefutable, and `Some v` is not — so the arm
//! is the probe, and it is also the position the fixture uses.

use redox_ast::{CaseArm, Expr, Item, Pattern, Program, Stmt};
use std::path::PathBuf;

fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixtures() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fmt")
}

const NOT_PARSEABLE: [&str; 1] = ["lexical.sdx"];

fn sources() -> Vec<PathBuf> {
  let mut found = Vec::new();
  for dir in [
    root().join("examples"),
    root().join("crates/redox-cli/tests/ui"),
    root().join("crates/redox-cli/tests/lint"),
  ] {
    let Ok(entries) = std::fs::read_dir(&dir) else {
      continue;
    };
    for entry in entries {
      let path = entry.expect("should read an entry").path();
      let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
      if path.extension().is_some_and(|x| x == "sdx") && !NOT_PARSEABLE.contains(&name) {
        found.push(path);
      }
    }
  }
  found.sort();
  assert!(found.len() > 20, "expected the corpora, found {found:?}");
  found
}

/// Every `case` arm in a program, in order.
///
/// Reached through the binding values, because a `case` is an expression and
/// the corpus puts every one of them on the right of a binding or an equation.
fn case_arms(program: &Program) -> Vec<&CaseArm> {
  let mut found = Vec::new();
  for expr in expressions(program) {
    if let Expr::Case { arms, .. } = expr {
      found.extend(arms.iter());
    }
  }
  found
}

/// The expressions a program's statements and equations hold, one level deep.
///
/// One level is enough for what this file needs: a `case` in the corpus is
/// always an equation's body or a binding's value, never buried in an
/// operand.
fn expressions(program: &Program) -> Vec<&Expr> {
  let mut found = Vec::new();
  for item in &program.items {
    if let Item::Fn(decl) = item {
      for clause in &decl.clauses {
        if let Some(block) = clause.body.plain() {
          for stmt in &block.stmts {
            found.extend(expression_of(stmt));
          }
        }
      }
    }
  }
  found
}

fn expression_of(stmt: &Stmt) -> Option<&Expr> {
  match &stmt.expr {
    Expr::Assert { value, .. } => Some(value),
    Expr::Binding(binding) => Some(&binding.value),
    Expr::Return { value, .. } => value.as_deref(),
    other if !other.is_statement_form() => Some(other),
    _ => None,
  }
}

/// Every pattern a program's `case` arms match on.
fn patterns(program: &Program) -> Vec<&Pattern> {
  case_arms(program)
    .into_iter()
    .map(|arm| &arm.pattern)
    .collect()
}

/// A pattern's debug form with spans blanked.
///
/// No `Paren` to strip, unlike leaf 3's comparison: a pattern's grouping is
/// not a node, so the parser leaves no record of it and the printer re-derives
/// it from `pattern_argument`. That makes the comparison exact, which is the
/// stronger statement of the two — the printer must reproduce the pattern
/// precisely, not merely equivalently.
///
/// # The scan is for a span, not for the word `span`
///
/// Leaf 3's version looked for the literal `span: (`, which was right for
/// `Expr` because every variant there is a struct variant with a named field.
/// `Pattern` has two **tuple** variants — `Wildcard(Span)` and `Rest(Span)` —
/// whose spans print as `Wildcard((496, 497))` with no field name, so that
/// scan left them in and the property reported that printing `_` had changed
/// the pattern.
///
/// So this matches the shape instead: a parenthesised pair of bare integers.
/// Nothing else in a `Pattern`'s debug form has it — a numeric literal prints
/// as a quoted `text` field — which is what makes the looser match safe.
fn meaning(pattern: &Pattern) -> String {
  let text = format!("{pattern:?}");
  let mut out = String::with_capacity(text.len());
  let mut rest = text.as_str();
  while let Some(at) = find_span(rest) {
    out.push_str(&rest[..at]);
    out.push('_');
    rest = &rest[at..];
    let close = rest.find(')').expect("find_span matched a closing paren");
    rest = &rest[close + 1..];
  }
  out.push_str(rest);
  out
}

/// Parse one pattern by putting it where a `case` arm's pattern goes.
fn reparse(text: &str) -> Option<Pattern> {
  // The signature takes a parameter because the clause does. A signature of
  // `()` beside a one-parameter clause does not parse, which cost a confusing
  // `n did not re-parse` on the simplest pattern in the corpus.
  let whole = format!("__rdx_probe : T -> i32\n__rdx_probe t = case t of\n  {text} -> 0\n");
  let program = redox_parser::parse_source(&whole).ok()?;
  patterns(&program).first().map(|p| (*p).clone())
}

/// Each form's canonical spelling, as a fixture pair.
#[test]
fn each_pattern_reaches_its_canonical_form() {
  let input = fixtures().join("patterns.sdx");
  let expected_path = fixtures().join("patterns.canonical.sdx");
  let source = std::fs::read_to_string(&input).expect("the fixture should exist");
  let program = redox_parser::parse_source(&source).expect("the fixture should parse");

  let lines: Vec<String> = patterns(&program)
    .into_iter()
    .map(redox_fmt::pattern_text)
    .collect();
  let printed = format!("{}\n", lines.join("\n"));

  if std::env::var_os("REDOX_BLESS").is_some() {
    std::fs::write(&expected_path, &printed).expect("should write the expectation");
    panic!("blessed {} — review the diff", expected_path.display());
  }

  let expected = std::fs::read_to_string(&expected_path).unwrap_or_else(|_| {
    panic!(
      "{} is missing; run with REDOX_BLESS=1 to create it",
      expected_path.display()
    )
  });
  assert_eq!(
    printed,
    expected,
    "the printed patterns differ from {}",
    expected_path.display()
  );
}

/// Printing a pattern does not change it, over every `case` arm in the
/// repository plus the fixture's own.
#[test]
fn printing_preserves_the_pattern() {
  let mut checked = 0;
  let mut paths = sources();
  paths.push(fixtures().join("patterns.sdx"));
  for path in paths {
    let source = std::fs::read_to_string(&path).expect("should read");
    let Ok(program) = redox_parser::parse_source(&source) else {
      continue;
    };
    for pattern in patterns(&program) {
      let printed = redox_fmt::pattern_text(pattern);
      let Some(reparsed) = reparse(&printed) else {
        panic!("{}: `{printed}` did not re-parse", path.display());
      };
      assert_eq!(
        meaning(pattern),
        meaning(&reparsed),
        "{}: printing `{printed}` changed the pattern",
        path.display()
      );
      let twice = redox_fmt::pattern_text(&reparsed);
      assert_eq!(
        printed,
        twice,
        "{}: printing `{printed}` is not idempotent",
        path.display()
      );
      checked += 1;
    }
  }
  // The fixture alone contributes more than twenty, so a count below that
  // means the traversal stopped finding arms rather than that the corpus is
  // thin. §10.8's point: an all-green corpus property proves less than it
  // looks if it read nothing.
  assert!(
    checked > 25,
    "only {checked} patterns were checked, which is too few to mean anything"
  );
}

/// Where the next `(N, M)` starts, if there is one.
///
/// A span and nothing else: both halves are bare digit runs, separated by a
/// comma and a space, inside parentheses.
fn find_span(text: &str) -> Option<usize> {
  let bytes = text.as_bytes();
  for at in 0..bytes.len() {
    if bytes[at] != b'(' {
      continue;
    }
    let mut i = at + 1;
    let first = digits(bytes, &mut i);
    if !first || !text[i..].starts_with(", ") {
      continue;
    }
    i += 2;
    if digits(bytes, &mut i) && bytes.get(i) == Some(&b')') {
      return Some(at);
    }
  }
  None
}

/// Advance past a run of digits, reporting whether there was one.
fn digits(bytes: &[u8], at: &mut usize) -> bool {
  let start = *at;
  while *at < bytes.len() && bytes[*at].is_ascii_digit() {
    *at += 1;
  }
  *at > start
}
