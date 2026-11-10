//! The committed `seed/redox1.wasm` is the blob `seed/SEED.md` claims it is.
//!
//! Plan 72 leaf 3. A checked-in binary is the one artifact in this repository
//! a reader cannot inspect, so the two things that *can* be asserted about it
//! are asserted: its size and its hash, against the table in `SEED.md` that
//! also records the commit and the command it came from.
//!
//! That makes a hand-edited blob and a stale table the same failure, which is
//! the point — a seed whose provenance row disagrees with its bytes is worse
//! than no row at all, because the row reads as verification.
//!
//! ```bash
//! REDOX_BLESS=1 cargo test -p redox-seed --test blob
//! ```
//!
//! # What this cannot assert
//!
//! That the blob is what the recipe *produces*. `wasm-opt` and rustc are
//! deterministic for a fixed input and a fixed version, but the versions are
//! pinned by `devenv.nix` rather than by this test, so a toolchain bump moves
//! the hash legitimately. The assertion is therefore about **agreement**
//! between the blob and its documentation, not about reproducibility — and
//! `tests/fixpoint.rs` is what checks the blob still *behaves*.

use std::path::PathBuf;

/// The workspace root, from this crate's manifest.
fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The `| blake3 | \`…\` |` and `| size | … bytes |` rows of `SEED.md`.
fn recorded(seed_md: &str) -> (Option<String>, Option<usize>) {
  let mut hash = None;
  let mut size = None;
  for line in seed_md.lines() {
    let line = line.trim();
    if let Some(rest) = line.strip_prefix("| blake3 |") {
      hash = Some(
        rest
          .trim()
          .trim_matches('|')
          .trim()
          .trim_matches('`')
          .to_owned(),
      );
    }
    if let Some(rest) = line.strip_prefix("| size |") {
      size = rest
        .trim()
        .trim_matches('|')
        .trim()
        .trim_end_matches(" bytes")
        .replace(',', "")
        .parse()
        .ok();
    }
  }
  (hash, size)
}

#[test]
fn the_committed_seed_matches_what_seed_md_records() {
  let root = workspace_root();
  let blob = root.join("seed/redox1.wasm");
  let doc_path = root.join("seed/SEED.md");

  let bytes = std::fs::read(&blob).expect(
    "seed/redox1.wasm should be committed — `moon run :seed-refresh` builds it, and \
     seed/SEED.md says why it exists",
  );
  let hash = blake3::hash(&bytes).to_hex().to_string();
  let doc = std::fs::read_to_string(&doc_path).expect("should read seed/SEED.md");

  if std::env::var("REDOX_BLESS").is_ok() {
    let mut out = String::new();
    for line in doc.lines() {
      if line.trim_start().starts_with("| blake3 |") {
        out.push_str(&format!("| blake3 | `{hash}` |\n"));
      } else if line.trim_start().starts_with("| size |") {
        out.push_str(&format!("| size | {} bytes |\n", bytes.len()));
      } else {
        out.push_str(line);
        out.push('\n');
      }
    }
    std::fs::write(&doc_path, out).expect("should write seed/SEED.md");
    panic!(
      "blessed seed/SEED.md — a changed hash is a changed compiler, so review the diff \
       rather than absorbing it. Also update the commit row by hand."
    );
  }

  let (want_hash, want_size) = recorded(&doc);
  assert_eq!(
    want_size,
    Some(bytes.len()),
    "seed/SEED.md records a different size than seed/redox1.wasm has"
  );
  assert_eq!(
    want_hash.as_deref(),
    Some(hash.as_str()),
    "seed/redox1.wasm does not hash to what seed/SEED.md records. Either the blob was \
     replaced without blessing, or the table is stale — and a provenance row that \
     disagrees with its bytes reads as verification while providing none."
  );
}
