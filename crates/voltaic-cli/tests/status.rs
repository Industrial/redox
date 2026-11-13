//! `docs/STATUS.md`, recomputed and compared.
//!
//! Plan 47 (`docs/41 - 47 - Status and Declines.md`) leaf 1. The document's
//! whole value is that a reader can trust its numbers without reading the
//! code, so they are **measured here and compared in the gate** rather than
//! typed by whoever last remembered.
//!
//! This is the shape the repository already uses three times — `tests/golden`,
//! `tests/ui`, `examples/*.stdout` — and for the same reason:
//!
//! ```bash
//! VOLTAIC_UPDATE=1 cargo test -p voltaic-cli --test status
//! ```
//!
//! rewrites the document and then **fails**, so a changed number reaches a
//! reviewer instead of a green build.
//!
//! The gap table is not written here. It is read out of `docs/GRAMMAR.md`'s
//! `UNREACHABLE` rows, because plan 46 built the inventory that knows the gaps
//! and a second list beside it would disagree within one row (plan 47 §1.3).

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Lines of `.rs` under `dir`, recursively.
fn rust_lines(dir: &Path) -> usize {
  let mut total = 0;
  let Ok(entries) = std::fs::read_dir(dir) else {
    return 0;
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if path.is_dir() {
      total += rust_lines(&path);
    } else if path.extension().is_some_and(|e| e == "rs") {
      total += std::fs::read_to_string(&path)
        .map(|s| s.lines().count())
        .unwrap_or(0);
    }
  }
  total
}

/// How many files in `dir` end with `suffix`.
fn count(dir: &Path, suffix: &str) -> usize {
  std::fs::read_dir(dir)
    .map(|entries| {
      entries
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(suffix))
        .count()
    })
    .unwrap_or(0)
}

/// Every crate under `crates/`, with its source and test line counts.
fn crates(root: &Path) -> Vec<(String, usize, usize)> {
  let mut out: Vec<(String, usize, usize)> = std::fs::read_dir(root.join("crates"))
    .expect("should read crates/")
    .flatten()
    .filter(|e| e.path().join("Cargo.toml").is_file())
    .map(|e| {
      let path = e.path();
      (
        path
          .file_name()
          .unwrap_or_default()
          .to_string_lossy()
          .into_owned(),
        rust_lines(&path.join("src")),
        rust_lines(&path.join("tests")),
      )
    })
    .collect();
  out.sort();
  out
}

/// The `UNREACHABLE` rows of `docs/GRAMMAR.md`: area, note, fixture.
///
/// Parsed rather than duplicated. Plan 46's inventory is the one place that
/// knows what cannot be written, and its fixtures are already asserted to
/// still fail — so a gap closing updates this document from that edit.
fn gaps(root: &Path) -> Vec<(String, String)> {
  let text = std::fs::read_to_string(root.join("docs/GRAMMAR.md")).expect("should read GRAMMAR.md");
  let mut out = Vec::new();
  for line in text.lines() {
    if !line.starts_with('|') {
      continue;
    }
    let cells = split_cells(line);
    if cells.len() < 3 || cells[1].trim().trim_matches('`') != "UNREACHABLE" {
      continue;
    }
    // The legend row explains the grade rather than grading an area; it has
    // no fixture path, which is what tells them apart.
    let note = cells[2..].join("|");
    let Some(fixture) = note
      .split('`')
      .find(|s| s.starts_with("tests/unreachable/"))
      .map(str::to_owned)
    else {
      continue;
    };
    out.push((cells[0].trim().to_owned(), fixture));
  }
  out.sort();
  out.dedup();
  out
}

/// Plan rows of doc 04, by status.
///
/// # Why this does not split the row into cells
///
/// The obvious parser — split on `|`, take the last cell — gets the wrong
/// answer, and got it three times before being written down. Register rows
/// quote Voltaic and Rust source in their scope text, and some of those quotes
/// contain a **literal pipe**: `` `|>` `` in row 2, `` `\|` `` in row 46. An
/// unescaped one is malformed markdown, so a renderer breaks the cell too, and
/// an escaped one still splits a naive parser.
///
/// So the status is found by **position among the status words** instead: a
/// row's status is the last status marker on the line, because the status cell
/// is last. That is robust to any number of pipes anywhere in the row, which a
/// cell split cannot be.
fn plan_counts(root: &Path) -> (usize, usize, usize) {
  let text = std::fs::read_to_string(root.join("docs/04 - Voltaic Plan of Plans.md"))
    .expect("should read the register");
  let (mut done, mut superseded, mut other) = (0, 0, 0);

  for line in text.lines() {
    if !is_register_row(line) {
      continue;
    }

    // A register row always carries a status, and that is what tells it from
    // doc 04's *other* tables — the `Plan | Document` index has the same
    // `| NN | `slug` |` opening and no status, and counting those four rows
    // put the total at 54 for a 50-row register.
    match row_status(line) {
      None => continue,
      Some("superseded") => superseded += 1,
      Some("done") => done += 1,
      Some(_) => other += 1,
    }
  }
  (done, superseded, other)
}

/// A markdown table row's cells, respecting `\|`.
///
/// The inventory quotes source that contains pipes, and an escaped one still
/// splits a naive parser — which silently dropped the or-patterns row from the
/// gap table and was only noticed because this generator counted the gaps and
/// came up one short of the document.
fn split_cells(line: &str) -> Vec<String> {
  const SENTINEL: char = '\u{0}';
  line
    .trim()
    .trim_matches('|')
    .replace("\\|", &SENTINEL.to_string())
    .split('|')
    .map(|cell| cell.replace(SENTINEL, "|"))
    .collect()
}

/// A register row's status: the **first** marker in its **last cell**.
///
/// Both halves are corrections, each made after the generator got a count
/// wrong, and the third one is why this is not simply `rfind`:
///
/// 1. Scanning the whole line counted doc 04's `Plan | Document` index as four
///    extra rows and totalled 54 for a register of 50 (plan 47 §10).
/// 2. Taking the *last* marker on the line fixed that, because a status cell
///    like row 44's opens `` `done` `` and then writes paragraphs that mention
///    other plans.
/// 3. But a status cell is **prose about statuses**, so the last marker in it
///    need not be its own. Plan 49's cell opens `` `done` `` and later says
///    *"rows 40–44 … gained a trailing `done`"* and *"this cell opens
///    `planned`"* — so `rfind` read the row as outstanding and `STATUS.md`
///    reported 44 done when 45 were (plan 49 §10.7).
///
/// The status **column** is the last one and a status **cell** opens with its
/// status, so first-marker-in-last-cell is the rule that matches how the
/// register is actually written.
fn row_status(line: &str) -> Option<&'static str> {
  first_status(status_cell(line))
}

/// Whether a line is a register row: `| NN | `slug` | …`.
///
/// Extracted so the counter and
/// [`every_register_row_opens_its_status_cell_with_its_status`] cannot drift.
/// They did: the test was written calling [`row_status`] directly, which
/// applies no shape check at all — it takes the last cell of *whatever* it is
/// given — and it matched this document's own prose, a numbered convention
/// reading *"Mark it `superseded` and name what replaced it"*. A rule about
/// statuses is not a status, and that is the third time that sentence has had
/// to be written in this file.
fn is_register_row(line: &str) -> bool {
  let Some(rest) = line.strip_prefix("| ") else {
    return false;
  };
  let Some((number, after)) = rest.split_once(" | ") else {
    return false;
  };
  !number.trim().is_empty()
    && number.trim().chars().all(|c| c.is_ascii_digit())
    && after.trim_start().starts_with('`')
}

/// The register's last column.
///
/// Cells are separated by `" | "`. An escaped pipe inside a cell is written
/// `` `\|` `` with no surrounding spaces — plan 47 §10 — so it does not split
/// here, which is the assumption this makes and the reason to state it.
fn status_cell(line: &str) -> &str {
  line
    .trim_end()
    .trim_end_matches('|')
    .trim_end()
    .rsplit(" | ")
    .next()
    .unwrap_or("")
}

fn first_status(cell: &str) -> Option<&'static str> {
  ["done", "superseded", "not-started", "planned", "partial"]
    .into_iter()
    .filter_map(|s| cell.find(&format!("`{s}`")).map(|at| (at, s)))
    .min_by_key(|(at, _)| *at)
    .map(|(_, s)| s)
}

/// Render the document.
fn render(root: &Path) -> String {
  let mut out = String::new();
  let cs = crates(root);
  let src: usize = cs.iter().map(|c| c.1).sum();
  let tests: usize = cs.iter().map(|c| c.2).sum();
  let (done, superseded, other) = plan_counts(root);

  out.push_str(
    "# Status\n\n\
     **Generated.** Every number here is measured from the working tree by\n\
     `crates/voltaic-cli/tests/status.rs` and compared in the gate, so a drifted\n\
     number fails the build. Rewrite it with:\n\n\
     ```bash\n\
     VOLTAIC_UPDATE=1 cargo test -p voltaic-cli --test status\n\
     ```\n\n\
     which rewrites and then **fails**, so the diff is reviewed rather than\n\
     absorbed into a green build.\n\n\
     Plan 47 (`docs/41 - 47 - Status and Declines.md`) built it. The posture is\n\
     doc 02's, taken from Emerald: *none of this is rounded up*.\n\n\
     ---\n\n## Plans\n\n",
  );
  let _ = writeln!(
    out,
    "| Status | Rows |\n|---|---:|\n| `done` | {done} |\n| `superseded` | {superseded} |\n| \
     outstanding | {other} |\n| **total** | **{}** |",
    done + superseded + other
  );
  out.push_str(
    "\n`docs/04 - Voltaic Plan of Plans.md` is the register and the authority. **`done` is a \
     register status, not a guarantee**: several rows record a later plan finding something \
     they got wrong, which is why the register carries the correction and this table only \
     counts.\n\n---\n\n## Source\n\n",
  );
  let _ = writeln!(out, "| Crate | `src` | `tests` |\n|---|---:|---:|");
  for (name, s, t) in &cs {
    let _ = writeln!(out, "| `{name}` | {s} | {t} |");
  }
  let _ = writeln!(out, "| **total** | **{src}** | **{tests}** |");

  out.push_str("\n---\n\n## Corpora\n\n| Corpus | Files | What it asserts |\n|---|---:|---|\n");
  let examples = count(&root.join("examples"), ".sdx");
  let golden = count(&root.join("tests/golden"), ".xrs");
  let ui = count(&root.join("crates/voltaic-cli/tests/ui"), ".sdx");
  let unreachable = count(&root.join("tests/unreachable"), ".sdx");
  let pairs = count(&root.join("benchmarks/pairs"), ".sdx");
  let _ = writeln!(
    out,
    "| `examples/*.sdx` | {examples} | the CI-checked source of truth for what works |\n\
     | `tests/golden/*.xrs` | {golden} | the generated Rust, as exact text |\n\
     | `crates/voltaic-cli/tests/ui/*.sdx` | {ui} | the diagnostic a bad program gets |\n\
     | `tests/unreachable/*.sdx` | {unreachable} | the gaps below, asserted to **still fail** |\n\
     | `benchmarks/pairs/*.sdx` | {pairs} | generated and hand-written Rust, as identical IR |"
  );
  let _ = writeln!(
    out,
    "\n**{examples} examples and {golden} goldens is not an error.** \
     `examples/lexical.sdx` deliberately does not parse — it is the error-path example, and \
     `benchmarks/REPORT.md` excludes it from the throughput sweep for the same reason. \
     Reporting \"{examples} examples, all green\" would be rounding up by exactly one file."
  );

  out.push_str("\n---\n\n## Gaps\n\n");
  let gs = gaps(root);
  let _ = writeln!(
    out,
    "{} Rust constructs a Voltaic program cannot express. Read out of `docs/GRAMMAR.md`'s \
     `UNREACHABLE` rows rather than listed again here, so the two cannot disagree; each has a \
     fixture that the gate asserts **still fails**, so closing a gap breaks the build until \
     both documents are updated.\n",
    gs.len()
  );
  let _ = writeln!(out, "| Construct | Fixture |\n|---|---|");
  for (area, fixture) in &gs {
    let _ = writeln!(out, "| {area} | `{fixture}` |");
  }
  out.push_str(
    "\nFull grading, with the nearest working alternative for each: \
     `docs/GRAMMAR.md`. Deliberate refusals are a different thing and live in \
     `docs/DECLINED.md`.\n",
  );

  out.push_str(
    "\n---\n\n## What this document does not say\n\n\
     - **No coverage percentage.** `moon run :coverage` produces one. A number here would \
     become a target, and it would mostly measure the emitter's match arms.\n\
     - **No test count as a quality measure.** The corpora above are counted because they are \
     corpora; a total would invite reading it as a score.\n\
     - **No roadmap and no dates.** What is left is the register's outstanding rows. When they \
     land is not a measurement.\n\
     - **No benchmark numbers.** `benchmarks/REPORT.md` has them, and plan 44 §1.2 \
     deliberately keeps them out of the gate because they are timings. Copying one here would \
     make this comparison flake.\n",
  );
  out
}

#[test]
fn status_is_current() {
  let root = workspace_root();
  let path = root.join("docs/STATUS.md");
  let rendered = render(&root);

  if std::env::var("VOLTAIC_UPDATE").is_ok() {
    std::fs::write(&path, &rendered).expect("should write STATUS.md");
    panic!(
      "updated docs/STATUS.md. The rewrite is deliberate and this failure is too: \
       review the diff rather than absorbing a changed number into a green build."
    );
  }

  let found = std::fs::read_to_string(&path).unwrap_or_default();
  assert_eq!(
    rendered, found,
    "docs/STATUS.md is out of date. Its numbers are measured from the working tree, so a \
     commit that changes them must update it: VOLTAIC_UPDATE=1 cargo test -p voltaic-cli --test \
     status"
  );
}

/// Every register row's status cell **opens** with its status.
///
/// `row_status`'s doc comment states this rule and three corrections that led
/// to it. What it did not do was enforce it, and an unenforced invariant in a
/// generated document is a number that can go wrong quietly.
///
/// Found by going wrong quietly. Row 65's cell was written with the outcome
/// text ahead of the status word, and that text contained `` `Newline \| Indent
/// \| Dedent` `` — a literal ` \| ` — which reassigned what `status_cell`
/// reads to a fragment of the prose. `first_status` then found `planned` later
/// in the line, counted a finished row as outstanding, and the gate passed.
///
/// That is the same hazard the comment above describes twice over: a status
/// cell is **prose about statuses**, so any marker in it that is not the first
/// one is somebody else's. Plan 47 §10 is the rule for the pipe — write it
/// `` `\|` `` with no surrounding spaces — and this test is what notices when
/// it is not followed.
#[test]
fn every_register_row_opens_its_status_cell_with_its_status() {
  const MARKERS: [&str; 5] = ["done", "superseded", "not-started", "planned", "partial"];

  let root = workspace_root();
  let text = std::fs::read_to_string(root.join("docs/04 - Voltaic Plan of Plans.md"))
    .expect("should read the register");

  let mut checked = 0;
  for line in text.lines() {
    if !is_register_row(line) {
      continue;
    }
    let Some(status) = row_status(line) else {
      continue;
    };
    let cell = status_cell(line).trim_start();

    // A status cell opens with its own marker, optionally emphasised.
    let opens = MARKERS.iter().any(|m| {
      let tick = format!("`{m}`");
      cell.starts_with(&tick) || cell.starts_with(&format!("**{tick}**"))
    });
    assert!(
      opens,
      "the status cell of\n  {}\ndoes not open with a status marker. It was read as \
       `{status}`, which is whichever marker happens to come first in\n  {}\n— and that is \
       a guess, not a status. Two ways this happens: the outcome text was written ahead of \
       the marker, or the row contains a literal ` | ` that moved the cell boundary (plan \
       47 §10: write an in-cell pipe as `\\|` with no surrounding spaces).",
      &line[..line.len().min(60)],
      &cell[..cell.len().min(80)],
    );
    checked += 1;
  }

  assert!(
    checked > 60,
    "only {checked} register rows were checked, which means the row predicate stopped \
     matching rather than that the register shrank"
  );
}

#[test]
fn the_gap_table_comes_from_the_inventory() {
  let root = workspace_root();
  let gs = gaps(&root);
  assert!(
    !gs.is_empty(),
    "no UNREACHABLE rows parsed out of docs/GRAMMAR.md — either the inventory lost its gap \
     rows or this parser stopped matching them, and both are worse than a wrong number"
  );
  for (area, fixture) in &gs {
    assert!(
      root.join(fixture).is_file(),
      "{area} cites {fixture}, which does not exist"
    );
  }
}
