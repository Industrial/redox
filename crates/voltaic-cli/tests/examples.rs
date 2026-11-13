//! Every example, held to all five of plan 04's properties.
//!
//! `docs/09 - 04 - Emitter Determinism.md` §3. One harness so that adding an
//! example adds five assertions rather than none — four examples existed with
//! two golden files before this plan, which is how coverage rots.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

/// `lexical.sdx` exercises the lexical surface (plan 02) and deliberately does
/// not parse — no grammar accepts most of its lines until later plans. A
/// declared exemption rather than an unexplained absence.
const NOT_PARSEABLE: [&str; 1] = ["lexical.sdx"];

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `.sdx` under `examples/`, sorted so failures report in a stable order.
fn examples() -> Vec<String> {
  let mut found: Vec<String> = std::fs::read_dir(workspace_root().join("examples"))
    .expect("examples/ should exist")
    .map(|e| e.expect("should read a directory entry").path())
    .filter(|p| p.extension().is_some_and(|x| x == "sdx"))
    .map(|p| {
      p.file_name()
        .expect("a file name")
        .to_string_lossy()
        .to_string()
    })
    .collect();
  found.sort();
  found
}

fn parseable() -> Vec<String> {
  examples()
    .into_iter()
    .filter(|e| !NOT_PARSEABLE.contains(&e.as_str()))
    .collect()
}

/// Criterion 1: an example with no golden expectation is a failure.
#[test]
fn every_example_has_a_golden_expectation() {
  let golden = workspace_root().join("tests/golden");
  let missing: Vec<String> = parseable()
    .into_iter()
    .filter(|e| !golden.join(Path::new(e).with_extension("xrs")).is_file())
    .collect();
  assert!(
    missing.is_empty(),
    "these examples have no golden file under tests/golden/; \
     run with VOLTAIC_UPDATE=1 to create them:\n  {}",
    missing.join("\n  ")
  );
}

/// Criterion 3: the emitter's output is already rustfmt-clean.
///
/// Doc 02 §5.3 forbids rustfmt at *emission* time; using it as a measurement
/// in a test is what turns doc 01 §3.4's "readable" into an assertion. If
/// rustfmt disagrees, the emitter is wrong, not this test.
///
/// Run with rustfmt's **defaults, outside the repository**: this repo's own
/// `rustfmt.toml` sets two-space indentation, which governs Voltaic's source and
/// not Voltaic's output. Generated code uses rustfmt's default four, so grading
/// it against the repo's config would compare the wrong things — a real
/// finding from plan 04's probe, not a hypothetical.
#[test]
fn generated_rust_is_rustfmt_clean() {
  if !support::rustfmt_available() {
    eprintln!(
      "SKIP generated_rust_is_rustfmt_clean: no usable rustfmt. \
       Readability is unverified in this environment."
    );
    return;
  }

  for example in parseable() {
    let case = support::transpile(&example);
    let formatted = support::rustfmt(&case.rust);
    assert_eq!(
      case.rust, formatted,
      "rustfmt would reformat the output for {example}; the emitter is wrong"
    );
  }
}

/// Criteria 2 and 5: golden match and byte-identical repetition.
#[test]
fn every_example_matches_its_golden_and_is_deterministic() {
  let golden_dir = workspace_root().join("tests/golden");
  let updating = std::env::var_os("VOLTAIC_UPDATE").is_some();
  let mut updated = Vec::new();

  for example in parseable() {
    let first = support::transpile(&example).rust.clone();
    let second = support::transpile(&example).rust.clone();
    assert_eq!(
      first, second,
      "{example} should transpile identically twice"
    );

    let expected_path = golden_dir.join(Path::new(&example).with_extension("xrs"));
    if updating {
      let changed = std::fs::read_to_string(&expected_path)
        .map(|prev| prev != first)
        .unwrap_or(true);
      std::fs::write(&expected_path, &first).expect("should write the expectation");
      if changed {
        updated.push(example);
      }
      continue;
    }

    let expected = std::fs::read_to_string(&expected_path)
      .unwrap_or_else(|_| panic!("missing {}", expected_path.display()));
    assert_eq!(first, expected, "golden mismatch for {example}");
  }

  // Criterion 7: an update rewrites and then fails, so a changed expectation is
  // reviewed rather than absorbed into a green build.
  assert!(
    updated.is_empty(),
    "updated {} expectation(s); review the diff and re-run without VOLTAIC_UPDATE",
    updated.len()
  );
}

/// Criterion 4: every parseable example compiles under rustc and runs.
#[test]
fn every_example_compiles_and_runs() {
  for example in parseable() {
    let case = support::transpile(&example);
    // `run` asserts zero rustc stderr and a zero exit status itself.
    case.run();
  }
}

/// Criterion 6: determinism's cause, not just its symptom.
///
/// Asserting byte-stability on today's examples would not catch a future
/// `HashMap` whose iteration order happens to be stable for small inputs.
#[test]
fn the_emitter_contains_no_unordered_collection() {
  let mut offenders = Vec::new();
  let mut files = Vec::new();
  collect_rs(
    &workspace_root().join("crates/voltaic-emit/src"),
    &mut files,
  );

  for path in files {
    let text = std::fs::read_to_string(&path).expect("should read a source file");
    for line in text.lines() {
      // Skip comments: the emitter's own doc comments explain *why* it uses no
      // `HashMap`, and matching the bare word flagged that prose as a
      // violation — found by running this check, not by reading it.
      let code = line.trim_start();
      if code.starts_with("//") {
        continue;
      }
      // Match usage rather than mention: a type position or a path.
      for needle in ["HashMap<", "HashMap::", "HashSet<", "HashSet::"] {
        if code.contains(needle) {
          offenders.push(format!("{} uses {needle}", path.display()));
        }
      }
    }
  }
  assert!(
    offenders.is_empty(),
    "the emitter must not use an unordered collection; its iteration order is \
     unspecified and output must be byte-stable:\n  {}",
    offenders.join("\n  ")
  );
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  for entry in entries {
    let path = entry.expect("should read a directory entry").path();
    if path.is_dir() {
      collect_rs(&path, out);
    } else if path.extension().is_some_and(|e| e == "rs") {
      out.push(path);
    }
  }
}

/// `lexical.sdx`'s exemption is real: it must still *lex*, and must still be
/// rejected by the parser, so the exemption cannot quietly hide a regression.
#[test]
fn the_exempt_example_lexes_but_does_not_parse() {
  let source = workspace_root().join("examples/lexical.sdx");
  let text = std::fs::read_to_string(&source).expect("should read lexical.sdx");
  assert!(
    voltaic_lexer_lexes(&text),
    "lexical.sdx must still lex; its exemption is from parsing only"
  );

  let out = Command::new(env!("CARGO_BIN_EXE_voltaic"))
    .arg("build")
    .arg(&source)
    .arg("-o")
    .arg(std::env::temp_dir().join("voltaic-exempt.xrs"))
    .output()
    .expect("should run the voltaic binary");
  assert!(
    !out.status.success(),
    "lexical.sdx is exempt because it does not parse; if it now parses, \
     remove the exemption and give it a golden file"
  );
}

/// Lexing is checked through the binary's own behaviour: a lex error reports
/// differently from a parse error, so a successful lex is observable without
/// depending on `voltaic-lexer` from this crate.
fn voltaic_lexer_lexes(text: &str) -> bool {
  let dir = std::env::temp_dir().join(format!("voltaic-lexprobe-{}", std::process::id()));
  std::fs::create_dir_all(&dir).expect("should create a scratch directory");
  let sdx = dir.join("probe.sdx");
  std::fs::write(&sdx, text).expect("should write the probe");
  let out = Command::new(env!("CARGO_BIN_EXE_voltaic"))
    .arg("build")
    .arg(&sdx)
    .arg("-o")
    .arg(dir.join("probe.xrs"))
    .output()
    .expect("should run the voltaic binary");
  std::fs::remove_dir_all(&dir).ok();
  // A lexical failure names the character or construct; a parse failure says
  // "expected". Only the latter is acceptable here.
  String::from_utf8_lossy(&out.stderr).contains("expected")
}

/// Plan 84 criterion 10: no `TokenKind`'s own name reaches a generated `.xrs`.
///
/// Plan 84 leaf 0 measured a bound clause the parser could not read being
/// swallowed into the first parameter's verbatim type, with the `=>` rendering
/// as the literal text `FatArrow` — `TokenKind::FatArrow`'s Debug name, in a
/// file doc 02 §10.1 calls a human-facing artifact. `?Sized` produced
/// `QuestionSized` the same way.
///
/// Leaf 1a refused the clause that caused those two, and `tests/ui/` pins that
/// one case. This is the wider assertion, and it is the one worth keeping: the
/// defect was never about `=>` specifically, it was about a token with no
/// source spelling reaching the output through a verbatim capture. A scan finds
/// the third instance; a fixture for the first two cannot.
///
/// Scoped to the names, not to the mechanism, because the mechanism is a
/// capture path and the names are what a reader would see.
#[test]
fn no_generated_rust_contains_a_token_kind_name() {
  const LEAKED: [&str; 2] = ["FatArrow", "QuestionSized"];
  let mut found = Vec::new();

  let mut targets: Vec<PathBuf> = Vec::new();
  let golden = workspace_root().join("tests/golden");
  if let Ok(entries) = std::fs::read_dir(&golden) {
    for entry in entries.flatten() {
      let path = entry.path();
      if path.extension().is_some_and(|e| e == "xrs") {
        targets.push(path);
      }
    }
  }
  targets.sort();

  for path in &targets {
    let Ok(text) = std::fs::read_to_string(path) else {
      continue;
    };
    // Comments are skipped, and `examples/binder.sdx` is why: comments are
    // trivia that reach the `.xrs` (doc 02 §10.1), and that example's own prose
    // explains the defect by naming the leaked token — so the first version of
    // this test failed on the file documenting what it checks for. A leak is
    // always in *code*, never in a comment, so the narrowing costs nothing.
    for line in text.lines() {
      if line.trim_start().starts_with("//") {
        continue;
      }
      for name in LEAKED {
        if line.contains(name) {
          found.push(format!("{} contains {name}", path.display()));
        }
      }
    }
  }

  assert!(
    !targets.is_empty(),
    "no goldens were scanned, so this test would pass by finding nothing"
  );
  assert!(
    found.is_empty(),
    "a TokenKind's Debug name reached generated Rust. The `.xrs` is a \
     human-facing artifact (doc 02 §10.1), so a token with no source spelling \
     in it is a defect in whatever captured it verbatim — not something to \
     bless:\n  {}",
    found.join("\n  ")
  );
}
