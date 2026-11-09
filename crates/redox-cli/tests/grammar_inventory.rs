//! `docs/GRAMMAR.md`'s citations, checked.
//!
//! Plan 46 (`docs/40 - 46 - Grammar Inventories.md`) leaf 2. Doc 02 §4.1 calls
//! the Rust axis the place *"where Redox's real risk lives"*, because an
//! `UNREACHABLE` construct is one a Redox program cannot write. An inventory
//! of that is only worth having if it is true, and plan 46 §1.3 measured how
//! easily it is not: **two thirds of a careless first pass was wrong** — eleven
//! of sixteen apparent gaps were the probe's own bug, each of which would have
//! sent a future row to implement something that already works.
//!
//! So every row of the inventory carries evidence, and this resolves it:
//!
//! 1. A `PASSTHROUGH` or `SUGARED` row names a corpus file, and it exists.
//! 2. An `UNREACHABLE` row names a fixture under `tests/unreachable/`, and it
//!    exists.
//! 3. **Every fixture still fails to compile.**
//!
//! The third is the one that matters, and it fails in the direction nothing
//! else in this suite does: when a later row *closes* a gap, the fixture starts
//! compiling and this test breaks. That is intended. The alternative is a
//! document still claiming a gap years after it closed, which is how an
//! inventory becomes worse than no inventory.

use std::path::{Path, PathBuf};
use std::process::Command;

mod support;

/// The inventory itself.
fn inventory() -> String {
  let path = workspace_root().join("docs/GRAMMAR.md");
  std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("should read {}: {e}", path.display()))
}

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// One graded row: its area, its grade, and the paths it cites.
struct Row {
  area: String,
  grade: String,
  citations: Vec<String>,
}

/// Every graded row of the inventory.
///
/// A row is a table line whose second cell is one of the three grades, which
/// is what distinguishes the inventory's tables from the prose tables around
/// them without needing a marker a future editor could forget.
fn rows(text: &str) -> Vec<Row> {
  let mut out = Vec::new();
  for line in text.lines() {
    if !line.starts_with('|') {
      continue;
    }
    let cells: Vec<&str> = line.trim().trim_matches('|').split('|').collect();
    if cells.len() < 3 {
      continue;
    }
    let grade = cells[1].trim().trim_matches('`').trim();
    if !matches!(grade, "PASSTHROUGH" | "SUGARED" | "UNREACHABLE") {
      continue;
    }
    // Citations are backticked paths anywhere in the row.
    let citations = cells[2..]
      .join("|")
      .split('`')
      .filter(|s| s.contains('/') && (s.ends_with(".sdx") || s.ends_with(".rs")))
      .map(str::to_owned)
      .collect();
    out.push(Row {
      area: cells[0].trim().to_owned(),
      grade: grade.to_owned(),
      citations,
    });
  }
  out
}

#[test]
fn the_inventory_grades_something() {
  let rows = rows(&inventory());
  assert!(
    rows.len() >= 30,
    "the inventory should grade every area of Rust's grammar; found {} rows",
    rows.len()
  );
}

#[test]
fn every_row_cites_evidence_that_exists() {
  let root = workspace_root();
  let mut missing = Vec::new();

  for row in rows(&inventory()) {
    if row.citations.is_empty() {
      missing.push(format!("{} ({}) cites nothing", row.area, row.grade));
      continue;
    }
    for cited in &row.citations {
      if !root.join(cited).is_file() {
        missing.push(format!("{} cites {cited}, which does not exist", row.area));
      }
    }
  }

  assert!(
    missing.is_empty(),
    "docs/GRAMMAR.md cites evidence that is not there. A row with no evidence \
     is prose, and plan 46 §1.3 is what that costs:\n  {}",
    missing.join("\n  ")
  );
}

#[test]
fn every_unreachable_fixture_still_fails() {
  if support::skip_without_rustc("every_unreachable_fixture_still_fails") {
    return;
  }
  let root = workspace_root();
  let mut closed = Vec::new();

  for row in rows(&inventory()) {
    if row.grade != "UNREACHABLE" {
      continue;
    }
    for cited in &row.citations {
      let path = root.join(cited);
      if !path.is_file() {
        continue; // reported by the citation test
      }
      if compiles(&path) {
        closed.push(format!("{} — {cited} now compiles", row.area));
      }
    }
  }

  assert!(
    closed.is_empty(),
    "these fixtures compile, so the gaps they record have CLOSED. This is good \
     news and a failing test: regrade the rows in docs/GRAMMAR.md and delete \
     the fixtures, because an inventory that claims a gap after it closed is \
     worse than none (plan 46 §3 criterion 3):\n  {}",
    closed.join("\n  ")
  );
}

/// Whether `redox check` accepts the file — rustc's verdict, not the parser's.
///
/// `check` rather than `build` on purpose: `or_pattern.sdx` **parses**, because
/// `|` is a guard in this surface, and fails only when rustc types it. A gap
/// list that asked the parser would have missed it.
fn compiles(path: &Path) -> bool {
  Command::new(env!("CARGO_BIN_EXE_redox"))
    .arg("check")
    .arg(path)
    .output()
    .is_ok_and(|out| out.status.success())
}
