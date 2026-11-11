//! Plan 91 criterion 1: the rewritten crates carry no comments at all.
//!
//! # Incremental, because the rewrite is thirteen waves
//!
//! A single end-state assertion would be red for the whole rewrite, so the
//! pre-commit gate would be red too and every wave would need a bypass. This
//! carries a list instead: [`REWRITTEN`] names the crates already rewritten and
//! is the thing each wave extends.
//!
//! # Two checks that check each other
//!
//! `HATCH_ALLOWED`'s shape in `port.rs`, for `HATCH_ALLOWED`'s reason. A list
//! plus a check that reads it can be narrow in two ways — a crate missing from
//! the list, or a name in the list that matches nothing — and both leave the
//! check green while it asserts less than it looks like it does.
//!
//! So [`every_name_in_the_list_is_a_ported_crate`] refuses a name that is not a
//! crate, and [`the_rewrite_is_complete`] is the end-state equality, `#[ignore]`d
//! until wave 13 and reporting what is left when run with `--ignored`.

mod support;

use std::path::{Path, PathBuf};

use support::ported;

/// The crates plan 91 has rewritten. Each wave appends one.
const REWRITTEN: &[&str] = &[
  "redox-diag",
  "redox-lexer",
  "redox-ast",
  "redox-parser",
  "redox-fmt",
  "redox-fuzz",
];

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sdx_files(dir: &Path, out: &mut Vec<PathBuf>) {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if path.is_dir() {
      sdx_files(&path, out);
    } else if path.extension().is_some_and(|e| e == "sdx") {
      out.push(path);
    }
  }
}

/// Every comment line in one crate's `.sdx`, as `path:line`.
///
/// A comment is decided per line and by its first non-space characters, which
/// is exactly how the lexer decides: `--` opens one, `-- |` opens a doc
/// comment, and both are comments for this purpose. A trailing `--` after code
/// is not matched here and not meant to be — plan 91 removes whole-line
/// commentary, and a trailing marker inside a string literal would be a false
/// positive this check must not have.
fn comment_lines(crate_name: &str) -> Vec<String> {
  let sdx = workspace_root().join("crates").join(crate_name).join("sdx");
  let mut files = Vec::new();
  sdx_files(&sdx, &mut files);
  files.sort();
  let mut found = Vec::new();
  for path in files {
    let Ok(text) = std::fs::read_to_string(&path) else {
      continue;
    };
    for (n, line) in text.lines().enumerate() {
      if line.trim_start().starts_with("--") {
        found.push(format!("{}:{}", path.display(), n + 1));
      }
    }
  }
  found
}

#[test]
fn every_rewritten_crate_is_comment_free() {
  let mut offenders = Vec::new();
  for crate_name in REWRITTEN {
    let found = comment_lines(crate_name);
    if !found.is_empty() {
      offenders.push(format!(
        "  {crate_name}: {} comment lines, first at {}",
        found.len(),
        found[0]
      ));
    }
  }
  assert!(
    offenders.is_empty(),
    "plan 91 criterion 1: a rewritten crate must hold no comment lines:\n{}",
    offenders.join("\n")
  );
}

/// A name in [`REWRITTEN`] that is not a ported crate would make the check
/// above pass by matching nothing.
#[test]
fn every_name_in_the_list_is_a_ported_crate() {
  let known: Vec<String> = ported().iter().map(|(c, _)| c.clone()).collect();
  let unknown: Vec<&&str> = REWRITTEN
    .iter()
    .filter(|c| !known.contains(&c.to_string()))
    .collect();
  assert!(
    unknown.is_empty(),
    "these names are in REWRITTEN but are not crates in seed/PORTED: {unknown:?}"
  );
}

/// Plan 91's end state. `#[ignore]`d until wave 13; run with `--ignored` to see
/// what is left.
#[test]
#[ignore = "red until plan 91's wave 13 rewrites the last crate"]
fn the_rewrite_is_complete() {
  let remaining: Vec<String> = ported()
    .iter()
    .map(|(c, _)| c.clone())
    .filter(|c| !REWRITTEN.contains(&c.as_str()))
    .collect();
  assert!(
    remaining.is_empty(),
    "{} of {} crates rewritten; remaining: {}",
    REWRITTEN.len(),
    ported().len(),
    remaining.join(", ")
  );
}
