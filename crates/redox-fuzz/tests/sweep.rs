//! Plan 42 §1.3: the deterministic sweep, and the row's value on a machine
//! with no fuzzer.
//!
//! §1.1 measured that there is no `cargo-fuzz` and no nightly here. So the
//! targets are exercised by this file instead — over a **fixed** input set, in
//! a fixed order, with a mutation schedule that is a function of an index and
//! nothing else.
//!
//! # No randomness, and that is a rule rather than a preference
//!
//! A gate test that fuzzes with a random seed fails on one commit in fifty for
//! a reason nobody can reproduce. Plan 23 §10.8, doc 30 §8.7 and plan 43 §10.3
//! are three plans' worth of evidence for what an unreproducible gate failure
//! costs — three rows wrote "unexplained" into their results before the cause
//! was found. Nothing here will add a fourth.
//!
//! So: no `rand`, no clock, no `HashMap` iteration order. Every input is
//! derived from the corpus by `mutate`, whose only argument is a position.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `.sdx` in the repository, as bytes, sorted by path.
///
/// `lexical.sdx` is **included** — it does not parse, which makes it the most
/// valuable seed here rather than the one to skip. Plan 43 criterion 3 and plan
/// 41 decision 5 made the same call for the same reason.
fn corpus() -> Vec<(String, Vec<u8>)> {
  let mut found: Vec<(String, Vec<u8>)> = Vec::new();
  for dir in [
    root().join("examples"),
    root().join("crates/redox-cli/tests/ui"),
    root().join("crates/redox-cli/tests/lint"),
  ] {
    let Ok(entries) = std::fs::read_dir(&dir) else {
      continue;
    };
    let mut paths: Vec<PathBuf> = entries
      .map(|entry| entry.expect("should read an entry").path())
      .filter(|path| path.extension().is_some_and(|x| x == "sdx"))
      .collect();
    paths.sort();
    for path in paths {
      let bytes = std::fs::read(&path).expect("should read a corpus file");
      found.push((path.display().to_string(), bytes));
    }
  }
  assert!(
    found.len() > 30,
    "expected the corpora, found {}",
    found.len()
  );
  found
}

/// Hand-written hostile inputs: the shapes a corpus of valid programs will
/// never contain.
///
/// Each one is here because it is a boundary rather than because it looks
/// nasty — the empty input, a lone newline, a byte that is not UTF-8, an
/// unclosed form of each bracketing construct, and an indentation that matches
/// no enclosing level.
fn hostile() -> Vec<(&'static str, Vec<u8>)> {
  vec![
    ("empty", Vec::new()),
    ("one newline", b"\n".to_vec()),
    ("only spaces", b"    ".to_vec()),
    // Not valid UTF-8: a lone continuation byte. `lex_source` is the only
    // target that can see this, and it is the reason it takes `&[u8]`.
    ("invalid utf-8", vec![0x80, 0x81, 0xfe, 0xff]),
    ("a lone nul", vec![0x00]),
    ("unterminated string", b"a :: ()\na = \"open\n".to_vec()),
    (
      "unterminated raw string",
      b"a :: ()\na = r#\"open\n".to_vec(),
    ),
    ("unterminated fence", b"{- open\na :: ()\n".to_vec()),
    ("unbalanced delimiter", b"a :: ()\na = (((\n".to_vec()),
    // Doc 03 §3.1's `RDX0002`: a dedent to a width matching no enclosing
    // level. The one input this file asserts is *rejected*.
    (
      "inconsistent dedent",
      b"main :: ()\nmain =\n    print 1\n  print 2\n".to_vec(),
    ),
    // Deep nesting, to find a recursion limit if there is one. Short enough to
    // stay fast and deep enough to blow a naive recursive descent.
    ("deep nesting", deep_nesting(200)),
  ]
}

/// `a = ((((…1…))))`, `depth` deep.
fn deep_nesting(depth: usize) -> Vec<u8> {
  let mut text = String::from("a :: ()\na = ");
  text.push_str(&"(".repeat(depth));
  text.push('1');
  text.push_str(&")".repeat(depth));
  text.push('\n');
  text.into_bytes()
}

/// One mutation of `seed`, chosen by `index` and by nothing else.
///
/// Four kinds, cycled so that every seed gets each: truncation, a byte flip, a
/// duplicated slice, and a splice of the seed with itself reversed. Crude by
/// design — the point is a large volume of **malformed** input, and a mutation
/// that produced valid programs would be testing the corpus again.
///
/// Deterministic is the whole requirement, so the arithmetic is deliberately
/// dull: `index` picks the kind and the position, and there is no state.
fn mutate(seed: &[u8], index: usize) -> Vec<u8> {
  if seed.is_empty() {
    return Vec::new();
  }
  let at = index.wrapping_mul(2_654_435_761) % seed.len();
  match index % 4 {
    // Truncate: finds anything that assumes a construct completes.
    0 => seed[..at].to_vec(),
    // Flip one bit: finds anything that assumes a byte's class.
    1 => {
      let mut out = seed.to_vec();
      out[at] ^= 1 << (index % 8);
      out
    }
    // Duplicate a slice: finds anything that assumes a thing appears once.
    2 => {
      let mut out = seed[..at].to_vec();
      out.extend_from_slice(&seed[at..]);
      out.extend_from_slice(&seed[at..]);
      out
    }
    // Splice with the reverse: produces structurally nonsensical input that
    // is still made of the language's own bytes, which is where a lexer's
    // assumptions show.
    _ => {
      let mut out = seed[..at].to_vec();
      out.extend(seed.iter().rev());
      out
    }
  }
}

/// Every input the sweep runs: the corpus, the hostile cases, and eight
/// mutations of each corpus file.
fn inputs() -> Vec<(String, Vec<u8>)> {
  let mut found: Vec<(String, Vec<u8>)> = Vec::new();
  for (name, bytes) in corpus() {
    for index in 0..8 {
      found.push((format!("{name} mutation {index}"), mutate(&bytes, index)));
    }
    found.push((name, bytes));
  }
  for (name, bytes) in hostile() {
    found.push((name.to_string(), bytes));
  }
  found
}

/// Criterion 2: the same inputs in the same order on every run.
///
/// Asserted rather than assumed, because "deterministic" is the property the
/// rest of this file rests on and the easiest one to lose — a `HashMap`
/// somewhere in the traversal would break it silently and the sweep would
/// still pass.
#[test]
fn the_input_set_is_deterministic() {
  let first = inputs();
  let second = inputs();
  assert_eq!(first.len(), second.len());
  for (a, b) in first.iter().zip(&second) {
    assert_eq!(a.0, b.0, "the order changed between runs");
    assert_eq!(a.1, b.1, "the bytes changed between runs for {}", a.0);
  }
  assert!(
    first.len() > 300,
    "the set should be large enough to be worth running, got {}",
    first.len()
  );
}

/// Criterion 3: no target panics on any input.
///
/// A product property rather than hygiene: plan 40's language server runs the
/// parser on every keystroke, so a panic on malformed input crashes an editor
/// in front of someone who has merely not finished typing.
///
/// A panic fails this test by unwinding out of it, which names the input in the
/// backtrace — so there is nothing to catch and nothing to report by hand.
#[test]
fn no_target_panics() {
  for (name, bytes) in inputs() {
    redox_fuzz::lex_source(&bytes);
    redox_fuzz::parse_program(&bytes);
    redox_fuzz::emit_program(&bytes);
    let _ = redox_fuzz::roundtrip(&bytes);
    // The name is used so a panic's surroundings are identifiable in the
    // output, and so the loop cannot be optimised into nothing.
    assert!(!name.is_empty());
  }
}

/// Criterion 4 and §1.4's invariant: **the emitter never produces invalid
/// Rust** for a program the parser accepted.
///
/// The row's centrepiece, and the oracle doc 04 calls *"a stronger oracle than
/// Emerald had"*. Emerald's targets assert *does not panic*, which finds
/// crashes and nothing else; this finds an emitter that quietly produces Rust
/// rustc will reject.
///
/// §1.4 states what it does not prove: `syn` parses where rustc parses **and**
/// checks, so this catches a mangled syntax and not a mangled program. That is
/// the right division — every semantic error rustc can find is one AGENTS.md
/// says rustc should find, while a syntax error in generated code is Redox's
/// own bug and nobody else's.
#[test]
fn the_generated_rust_always_parses() {
  let mut checked = 0;
  for (name, bytes) in inputs() {
    let Some(rust) = redox_fuzz::roundtrip(&bytes) else {
      continue;
    };
    if let Err(error) = syn::parse_file(&rust) {
      panic!(
        "{name}: the parser accepted the input and the emitter produced Rust \
         that does not parse: {error}\n\n{rust}"
      );
    }
    checked += 1;
  }
  // The corpus alone contributes thirty, so a count below that means the
  // traversal stopped reaching the parser rather than that the inputs are
  // thin. Plan 34 §10.8's point: an all-green corpus property proves less than
  // it looks if it read nothing.
  assert!(
    checked > 30,
    "only {checked} inputs reached the emitter, which is too few to mean \
     anything"
  );
}

/// Criterion 5: the negative case, so criterion 3 is not vacuous.
///
/// A sweep that only asserts *no panic* over inputs that happen to parse
/// proves nothing about rejection. Plan 43 §10.1 and plan 41's criterion 4
/// record the same discipline, and plan 43 records getting it backwards once:
/// its first negative case used a *deeper* line, which lexes correctly, when
/// `RDX0002` is about a **dedent** landing between two levels.
#[test]
fn an_input_that_must_be_rejected_is_rejected() {
  let bad = b"main :: ()\nmain =\n    print 1\n  print 2\n";
  assert!(
    redox_lexer::lex(&String::from_utf8_lossy(bad)).is_err(),
    "a dedent matching no enclosing level must be RDX0002"
  );
  assert!(
    redox_fuzz::roundtrip(bad).is_none(),
    "and nothing that fails to lex should reach the emitter"
  );
  // The positive counterpart, so the case is not passing for an unrelated
  // reason.
  let good = b"main :: ()\nmain =\n  if c\n    print 1\n  print 2\n";
  assert!(
    redox_lexer::lex(&String::from_utf8_lossy(good)).is_ok(),
    "a nested block then a dedent to a level that *is* open must lex"
  );
}
