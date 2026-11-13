//! `docs/DECLINED.md` against doc 03 §22.
//!
//! Plan 47 (`docs/41 - 47 - Status and Declines.md`) leaf 2. §22 is the source
//! and will gain rows; `DECLINED.md` restates them for a reader who has not
//! read a 2,000-line surface document. A decline recorded in only one of the
//! two is a decline nobody can find, so this asserts every §22 construct
//! appears in the document.
//!
//! It checks **inclusion, not equality**. `DECLINED.md` deliberately says more
//! than §22 does — it groups the declines by the admission test each one fails,
//! and it carries a "not declined, merely absent" section distinguishing a
//! decline from a gap, which is the distinction that makes the document worth
//! having beside `docs/GRAMMAR.md`.

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The construct named by each row of doc 03 §22's table.
fn declines_in_the_specification() -> Vec<String> {
  let text = std::fs::read_to_string(workspace_root().join("docs/03 - Voltaic Surface Design.md"))
    .expect("should read doc 03");

  let section = text
    .split("## 22. Declines")
    .nth(1)
    .expect("doc 03 should have a §22")
    .split("\n## ")
    .next()
    .expect("§22 should end");

  section
    .lines()
    .filter(|l| l.starts_with("| "))
    .filter_map(|l| l.trim().trim_matches('|').split('|').next())
    .map(str::trim)
    .filter(|c| !c.is_empty() && *c != "Construct" && !c.starts_with("---"))
    .map(str::to_owned)
    .collect()
}

#[test]
fn the_specification_still_has_declines_to_check() {
  let declines = declines_in_the_specification();
  assert!(
    declines.len() >= 15,
    "doc 03 §22 should hold the declines; parsed only {}: {declines:?}",
    declines.len()
  );
}

#[test]
fn every_declined_construct_is_documented() {
  let declined = std::fs::read_to_string(workspace_root().join("docs/DECLINED.md"))
    .expect("should read docs/DECLINED.md");

  let missing: Vec<String> = declines_in_the_specification()
    .into_iter()
    .filter(|construct| !declined.contains(construct.as_str()))
    .collect();

  assert!(
    missing.is_empty(),
    "doc 03 §22 declines these and docs/DECLINED.md does not mention them. A decline \
     recorded in one place only is a decline nobody can find:\n  {}",
    missing.join("\n  ")
  );
}
