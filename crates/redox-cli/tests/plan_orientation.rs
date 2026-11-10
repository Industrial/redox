//! A plan's orientation has to reach the plan.
//!
//! Most plan documents carry a §11 *"Corrections from orientation"*: findings
//! measured against the real code after §1–§5 were written. Several of those
//! findings **refute** a criterion in the same document's §5, or a leaf in its
//! §4 — that is what an orientation is for.
//!
//! A refutation that is written down and not acted on is worse than no
//! orientation at all. The row gets built from §5, against something its own
//! plan already disproved, and the §11 that disproved it reads as agreement
//! because nobody looked.
//!
//! # Why this is a test and not a convention
//!
//! Measured 2026-11-10 across plans 56–64, every one of which has a §11:
//! **only plan 56 had fed its findings back.** Eight documents out of nine.
//!
//! The clearest was plan 59, whose §11.2 is titled *"Criterion 5 rejects a
//! program that compiles today"* — with the measurement, `const SQ : u32 =
//! square 7`, which compiles clean and which criterion 5 refused because it
//! contains an application. Criterion 5 stood verbatim, and it also broke
//! AGENTS.md's *"every check rustc can make, rustc makes"*.
//!
//! So this is the same kind of gate as `status.rs` and `declined.rs`: a
//! documentation invariant that has already failed in practice, asserted
//! rather than hoped for.
//!
//! # What it checks, and what it cannot
//!
//! It checks that a document with a §11 **cites §11 from its own §1–§9** — the
//! plan proper, before the orientation section. That is a weak proxy for "the
//! findings were acted on" and it is deliberately weak: whether an amendment
//! is *correct* is not mechanically decidable, and a test that pretended
//! otherwise would be the hollow kind this project has caught itself writing
//! before.
//!
//! What makes the weak version worth having is that the failure it caught was
//! not subtle. Eight documents cited §11 **zero** times. A document whose §1–§9
//! never mention their own orientation has not been reconciled with it, whatever
//! else may be true.

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every numbered plan document: `docs/NN - MM - Title.md`.
///
/// Read off the filesystem rather than from a list, so a new plan is covered
/// the moment it is written. `docs/04 - Redox Plan of Plans.md` and the
/// unnumbered documents do not match the shape and are skipped.
fn plan_documents(root: &Path) -> Vec<PathBuf> {
  let mut found: Vec<PathBuf> = std::fs::read_dir(root.join("docs"))
    .expect("should read docs/")
    .filter_map(|entry| {
      let path = entry.expect("should read a docs/ entry").path();
      let name = path.file_name()?.to_str()?;
      // `NN - MM - …`: two numbers separated by " - ". The register is
      // `NN - Title` with one, so it does not match.
      let mut parts = name.split(" - ");
      let first = parts.next()?;
      let second = parts.next()?;
      let is_plan = first.chars().all(|c| c.is_ascii_digit())
        && !first.is_empty()
        && second.chars().all(|c| c.is_ascii_digit())
        && !second.is_empty();
      is_plan.then_some(path)
    })
    .collect();
  found.sort();
  found
}

#[test]
fn every_orientation_is_cited_by_the_plan_it_corrects() {
  let root = workspace_root();
  let docs = plan_documents(&root);
  assert!(
    docs.len() > 40,
    "only {} plan documents matched the `NN - MM - Title.md` shape, which means this \
     test stopped finding them rather than that the project shrank",
    docs.len()
  );

  let mut oriented = 0;
  let mut unreconciled = Vec::new();

  for path in &docs {
    let text = std::fs::read_to_string(path)
      .unwrap_or_else(|e| panic!("should read {}: {e}", path.display()));

    // No §11 is fine: a plan whose orientation found nothing worth a section,
    // or one written before the convention, has nothing to reconcile.
    let Some(at) = text.find("\n## 11.") else {
      continue;
    };
    oriented += 1;

    // The plan proper is everything before the orientation section.
    if !text[..at].contains("§11") {
      unreconciled.push(
        path
          .file_name()
          .expect("a plan document has a file name")
          .to_string_lossy()
          .into_owned(),
      );
    }
  }

  assert!(
    oriented > 5,
    "only {oriented} documents have a §11, so this test is not measuring what it claims"
  );

  assert!(
    unreconciled.is_empty(),
    "{} plan document(s) carry a §11 their own §1–§9 never cite, so the orientation has \
     not been reconciled with the plan:\n  {}\n\nEach §11 subsection that refutes, narrows \
     or re-sizes a criterion or a leaf should be carried into §4 or §5 — quoting the \
     original wording and naming the subsection, as `docs/50 - 56 - Currying.md` does for \
     its withdrawn leaf 3 and its restated criteria 5 and 6. A refutation left only in §11 \
     means the row is built from criteria its own plan disproved.",
    unreconciled.len(),
    unreconciled.join("\n  "),
  );
}
