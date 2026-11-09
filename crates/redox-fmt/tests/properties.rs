//! Plan 34 §1.4's two properties, and §1.5's inversion — over every `.sdx` in
//! the repository.
//!
//! These are worth more than any golden. A golden asserts one output and says
//! nothing about either property; these say what the formatter is actually
//! *for*:
//!
//! 1. **Idempotence** — `fmt(fmt(x)) == fmt(x)`. A formatter that oscillates
//!    is maddening to diagnose from a CI diff.
//! 2. **Meaning preservation** — `parse(fmt(x))` is the same program as
//!    `parse(x)`. This is the one Emerald's plan 90 lost: its formatter
//!    re-added parentheses it could not know were redundant, so *"example code
//!    rewritten in the terser style would be silently reverted by the same CI
//!    gate that checked it."*
//! 3. **No text lost** — every comment and attribute survives. Doc 02 §3.2
//!    records Emerald's formatter losing every non-doc comment, and for Redox
//!    that would be a product defect, since the same trivia must reach the
//!    `.xrs`.
//!
//! And §1.5: `--check` passing over `examples/` means the hand-written corpus
//! **is** the style guide, rather than the formatter being measured against
//! hand-written expectations.
//!
//! `lexical.sdx` is exempt, by the same declared exemption `examples.rs`
//! carries: it exercises the lexical surface and deliberately does not parse.

use std::path::{Path, PathBuf};

/// Plan 02's lexer fixture, which deliberately does not parse. The same
/// exemption `examples.rs` declares, named here rather than silently skipped.
const NOT_PARSEABLE: [&str; 1] = ["lexical.sdx"];

fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `.sdx` in the repository that is meant to parse — the examples, the
/// UI corpus and the lint corpus together.
///
/// The UI corpus is included deliberately even though its cases are *invalid*
/// programs: a case that fails to parse is skipped by `parse_source`, and one
/// that parses and then fails a later check is exactly the kind of odd input a
/// formatter should survive.
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
      if path.extension().is_some_and(|x| x == "sdx")
        && !NOT_PARSEABLE.contains(&path.file_name().unwrap().to_str().unwrap())
      {
        found.push(path);
      }
    }
  }
  found.sort();
  assert!(found.len() > 20, "expected the corpora, found {found:?}");
  found
}

/// Format one file, or `None` if it does not parse — which is not a failure
/// here: `tests/ui/` is full of programs that must not.
fn formatted(path: &Path) -> Option<(String, String)> {
  let source = std::fs::read_to_string(path).expect("should read the case");
  let program = redox_parser::parse_source(&source).ok()?;
  let once = redox_fmt::format(&program, &source);
  Some((source, once))
}

#[test]
fn formatting_is_idempotent() {
  for path in sources() {
    let Some((_, once)) = formatted(&path) else {
      continue;
    };
    let reparsed = redox_parser::parse_source(&once).unwrap_or_else(|e| {
      panic!(
        "{} did not re-parse after formatting: {e:?}",
        path.display()
      )
    });
    let twice = redox_fmt::format(&reparsed, &once);
    assert_eq!(
      once,
      twice,
      "formatting {} is not idempotent",
      path.display()
    );
  }
}

#[test]
fn formatting_preserves_meaning() {
  for path in sources() {
    let Some((source, once)) = formatted(&path) else {
      continue;
    };
    let before = redox_parser::parse_source(&source).expect("parsed once already");
    let after = redox_parser::parse_source(&once)
      .unwrap_or_else(|e| panic!("{} did not re-parse: {e:?}", path.display()));

    // Spans move when whitespace does, so the comparison is of the printed
    // debug form with spans stripped. Crude, and it compares *everything
    // else* — which is what "the same program" has to mean for a formatter.
    assert_eq!(
      without_spans(&before),
      without_spans(&after),
      "formatting {} changed the program",
      path.display()
    );
  }
}

/// A program's debug form with every span blanked.
///
/// Single-line `{:?}` rather than `{:#?}`, and a scan rather than a line
/// filter — because a pretty-printed span is **four lines**:
///
/// ```text
/// span: (
///     613,
///     640,
/// ),
/// ```
///
/// so dropping the lines that begin `span:` left the numbers behind and the
/// comparison still saw them. The first version of this test failed for that
/// reason and not because any program had changed, which is worth recording:
/// a property test whose normaliser is wrong reports a defect in the subject.
fn without_spans(program: &redox_ast::Program) -> String {
  let text = format!("{program:?}");
  let mut out = String::with_capacity(text.len());
  let mut rest = text.as_str();
  while let Some(at) = rest.find("span: (") {
    out.push_str(&rest[..at]);
    out.push_str("span: _");
    rest = &rest[at + "span: (".len()..];
    // A span holds two integers and no nested parentheses, so the next `)`
    // closes it.
    match rest.find(')') {
      Some(close) => rest = &rest[close + 1..],
      None => break,
    }
  }
  out.push_str(rest);
  out
}

#[test]
fn formatting_loses_no_comments() {
  for path in sources() {
    let Some((source, once)) = formatted(&path) else {
      continue;
    };
    // Counted on the source text rather than the AST, so a comment the
    // formatter failed to attach anywhere is still counted as lost.
    let before = source.matches("--").count();
    let after = once.matches("--").count();
    assert_eq!(
      before,
      after,
      "formatting {} changed the number of `--` markers, so text moved or vanished",
      path.display()
    );
  }
}

/// Plan 34 §1.5: the corpus is the style guide.
#[test]
fn every_example_is_already_canonical() {
  let mut unformatted = Vec::new();
  for path in sources() {
    if !path.to_string_lossy().contains("/examples/") {
      continue;
    }
    let Some((source, once)) = formatted(&path) else {
      continue;
    };
    if source != once {
      unformatted.push(path.display().to_string());
    }
  }
  assert!(
    unformatted.is_empty(),
    "these examples are not canonical; run `redox fmt` on them and review the diff:\n{}",
    unformatted.join("\n")
  );
}
