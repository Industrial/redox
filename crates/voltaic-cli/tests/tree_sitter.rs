//! Plan 41 §1.3: the grammar's real assertion.
//!
//! `tree-sitter test`'s corpus files are hand-written expectations, and doc 02
//! §9.4 wants the grammar to be *"an independent check on the grammar's
//! shape"*. Hand-written s-expressions are not that — they check that the
//! grammar does what its author last thought it did, and this row watched that
//! happen: four of the five corpus cases were written by hand and four of them
//! were wrong about the tree.
//!
//! The assertion with teeth is **every `.sdx` that is meant to build parses
//! with no `ERROR` node**. The corpus already covers every construct the
//! language has, it is maintained by the compiler's own gate, and a grammar
//! that drifts from the language fails without anyone writing a new
//! expectation. The third row to reach for the existing corpus on that
//! reasoning, after plan 34's properties and plan 43's.
//!
//! # Why this shells out
//!
//! Plan 41 decision 6. Linking the `tree-sitter` crate would compile the
//! generated parser in every `cargo check` to serve one test, and the CLI is
//! the same one the editors will use — so the test exercises the thing that
//! ships rather than a second binding to it.
//!
//! The CLI is pinned in `devenv.nix` by leaf 1, which is what makes a test
//! that shells out to it reproducible.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn grammar_dir() -> PathBuf {
  root().join("tree-sitter-voltaic")
}

/// Parse one file, reporting whether tree-sitter found no `ERROR`.
///
/// `--quiet` suppresses the tree; the exit code is the verdict. AGENTS.md's
/// own gotcha applies — *check exit codes, not output text* — and here it is
/// also the simpler read.
///
/// The working directory is the grammar's, because the CLI finds a grammar by
/// looking at where it was invoked. Discovered by running it from the
/// repository root and getting *"No language found"* for every file, which
/// reads as a broken grammar and is a broken invocation.
fn parses_cleanly(path: &Path) -> bool {
  let output = Command::new("tree-sitter")
    .current_dir(grammar_dir())
    .arg("parse")
    .arg("--quiet")
    .arg(path)
    .output()
    .unwrap_or_else(|e| panic!("tree-sitter should be on PATH, pinned by devenv.nix: {e}"));
  output.status.success()
}

/// Every `.sdx` in a directory, sorted.
fn sources(dir: &Path) -> Vec<PathBuf> {
  let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
    .unwrap_or_else(|e| panic!("should read {}: {e}", dir.display()))
    .map(|entry| entry.expect("should read an entry").path())
    .filter(|path| path.extension().is_some_and(|x| x == "sdx"))
    .collect();
  found.sort();
  found
}

/// Criterion 3: every example parses with no `ERROR`, `lexical.sdx` included.
///
/// That one file is exempt from `examples.rs` and from the formatter's
/// properties, because it does not *parse* — and it is included here for plan
/// 43 criterion 3's reason: the exemption is about the **compiler's** parser,
/// a highlighting grammar should survive the file, and an independent check is
/// worth least on the inputs the compiler already accepts.
#[test]
fn every_example_parses_with_no_error() {
  let examples = sources(&root().join("examples"));
  assert!(
    examples.len() > 20,
    "expected the corpus, found {}",
    examples.len()
  );
  let failing: Vec<String> = examples
    .iter()
    .filter(|path| !parses_cleanly(path))
    .map(|path| path.display().to_string())
    .collect();
  assert!(
    failing.is_empty(),
    "the grammar found an ERROR in these, which means it has drifted from the \
     language:\n{}",
    failing.join("\n")
  );
}

/// The lint corpus too, for the same reason: plan 38 made those cases
/// *programs that build*, differing from an example only in carrying a
/// warning.
#[test]
fn every_lint_case_parses_with_no_error() {
  let cases = sources(&root().join("crates/voltaic-cli/tests/lint"));
  assert!(!cases.is_empty(), "the lint corpus should exist");
  for path in cases {
    assert!(
      parses_cleanly(&path),
      "the grammar found an ERROR in {}",
      path.display()
    );
  }
}

/// Criterion 4: the negative case, so criterion 3 is not vacuous.
///
/// A grammar whose scanner accepted any indentation would pass the sweep and
/// be wrong in the one place this row is meant to be precise. Plan 43 §10.1
/// records getting this exactly backwards once — its first negative case used
/// a *deeper* line, which lexes correctly, when `RDX0002` is about a **dedent**
/// landing between two levels.
#[test]
fn an_inconsistent_dedent_is_an_error() {
  let dir = std::env::temp_dir().join("voltaic-tree-sitter-negative");
  std::fs::create_dir_all(&dir).expect("should create a scratch directory");
  let path = dir.join("inconsistent.sdx");
  // Opens at four, then dedents to two, which matches neither 0 nor 4.
  std::fs::write(
    &path,
    "main : ()\nmain =\n    println! \\-> \"#{1}\"\n  println! \\-> \"#{2}\"\n",
  )
  .expect("should write the case");
  assert!(
    !parses_cleanly(&path),
    "a dedent matching no enclosing level must produce an ERROR, or the sweep \
     above proves nothing"
  );
  // And its positive counterpart, so the case is not failing for an unrelated
  // reason — the other half of plan 43 §10.1's correction.
  let nested = dir.join("nested.sdx");
  std::fs::write(
    &nested,
    "main : ()\nmain =\n  if c\n    println! \\-> \"#{1}\"\n  println! \\-> \"#{2}\"\n",
  )
  .expect("should write the case");
  assert!(
    parses_cleanly(&nested),
    "a nested block then a dedent to a level that *is* open must parse"
  );
}

/// The grammar's own corpus runs, because `parse` cannot see what it checks.
///
/// Added at plan 65. `parses_cleanly` above asks `tree-sitter parse` for an
/// `ERROR`, and that is a weaker question than it looks: a **hidden** rule
/// that is missing — `expression_statement` is `seq($._expression,
/// $._newline)`, and `_newline` is hidden — produces a `(MISSING _newline)`
/// that `parse` neither prints nor exits nonzero on. The grammar was wrong in
/// exactly that way during this row and four readers agreed it was fine: the
/// hand-written lexer, the parser, `voltaic fmt` and `tree-sitter parse`.
///
/// `tree-sitter test` is the one that caught it, and until now nothing ran it.
#[test]
fn the_grammars_own_corpus_passes() {
  let output = Command::new("tree-sitter")
    .current_dir(grammar_dir())
    .arg("test")
    .output()
    .expect("tree-sitter should be on PATH, pinned by devenv.nix");
  assert!(
    output.status.success(),
    "`tree-sitter test` should pass in {}:\n{}{}",
    grammar_dir().display(),
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr),
  );
}

/// Criterion 1: the committed parser was generated by the pinned CLI at ABI 15.
///
/// §1.1 found that a grammar with no `tree-sitter.json` is **silently**
/// generated against ABI 14, which is the kind of thing that is invisible
/// until an editor loads the parser and says nothing useful. Asserted on the
/// generated file rather than by re-running `generate`, because the question
/// is about the artefact that ships.
#[test]
fn the_generated_parser_is_abi_15() {
  let parser = grammar_dir().join("src/parser.c");
  let text = std::fs::read_to_string(&parser)
    .unwrap_or_else(|e| panic!("should read {}: {e}", parser.display()));
  assert!(
    text.contains("#define LANGUAGE_VERSION 15"),
    "the committed parser should be ABI 15; `tree-sitter.json` is what makes \
     `generate` use it"
  );
  assert!(
    grammar_dir().join("tree-sitter.json").is_file(),
    "and the file that asks for it should be committed"
  );
}

/// Criterion 5's half that can be checked here: `highlights.scm` loads and
/// matches something.
///
/// A query naming a node type the grammar does not have is an **error**, not a
/// no-op, which is how `self` was found in it — the grammar spells `self` and
/// `Self` as one `self_expression` token rather than as two keywords. So
/// running the query at all is a real check on the pair.
#[test]
fn the_highlight_query_loads_and_matches() {
  let output = Command::new("tree-sitter")
    .current_dir(grammar_dir())
    .arg("query")
    .arg("queries/highlights.scm")
    .arg(root().join("examples/hello.sdx"))
    .output()
    .expect("tree-sitter should be on PATH");
  assert!(
    output.status.success(),
    "the highlight query should load: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let text = String::from_utf8_lossy(&output.stdout);
  assert!(
    text.contains("capture:"),
    "and it should match something in the smallest example: {text}"
  );
}
