//! Golden tests: `.sdx` beside its expected `.xrs`, compared as exact text.
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`), leaf 8.
//!
//! Golden text proves **stability**; `hello.rs`'s compile-and-run proves
//! **correctness**. Doc 02 §8.1-8.2 is explicit that neither substitutes for
//! the other, which is why both exist from the first plan. Doc 01 §3.4's
//! determinism requirement is what makes exact-text comparison possible at
//! all.
//!
//! To regenerate after an intentional emitter change:
//!
//! ```bash
//! REDOX_BLESS=1 cargo test -p redox-cli --test golden
//! ```
//!
//! Blessing rewrites the expectation files and then **fails the run**, so a
//! blessed diff is always reviewed rather than silently absorbed into a green
//! build.

use std::path::PathBuf;
use std::process::Command;

fn golden_dir() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden")
}

fn blessing() -> bool {
  std::env::var_os("REDOX_BLESS").is_some()
}

/// Every `.sdx` under `tests/golden/` is a case; its expectation is the
/// sibling `.xrs`. Adding a fixture file is all it takes to add a case.
fn cases() -> Vec<PathBuf> {
  let mut found: Vec<PathBuf> = std::fs::read_dir(golden_dir())
    .expect("tests/golden should exist")
    .map(|e| e.expect("should read a directory entry").path())
    .filter(|p| p.extension().is_some_and(|e| e == "sdx"))
    .collect();
  // Sorted so the reported order is stable across filesystems.
  found.sort();
  found
}

fn transpile(sdx: &PathBuf) -> String {
  let tmp = std::env::temp_dir().join(format!(
    "redox-golden-{}-{}.xrs",
    sdx.file_stem().expect("a stem").to_string_lossy(),
    std::process::id()
  ));
  let out = Command::new(env!("CARGO_BIN_EXE_redox"))
    .arg("build")
    .arg(sdx)
    .arg("-o")
    .arg(&tmp)
    .output()
    .expect("should run the redox binary");
  assert!(
    out.status.success(),
    "redox build failed on {}:\n{}",
    sdx.display(),
    String::from_utf8_lossy(&out.stderr)
  );
  let text = std::fs::read_to_string(&tmp).expect("should read the generated .xrs");
  std::fs::remove_file(&tmp).ok();
  text
}

#[test]
fn golden_output_matches() {
  let cases = cases();
  assert!(
    !cases.is_empty(),
    "there should be at least one golden case"
  );

  let mut blessed = Vec::new();
  for sdx in &cases {
    let expected_path = sdx.with_extension("xrs");
    let actual = transpile(sdx);

    if blessing() {
      let changed = std::fs::read_to_string(&expected_path)
        .map(|prev| prev != actual)
        .unwrap_or(true);
      std::fs::write(&expected_path, &actual).expect("should write the expectation");
      if changed {
        blessed.push(expected_path.clone());
      }
      continue;
    }

    let expected = std::fs::read_to_string(&expected_path).unwrap_or_else(|_| {
      panic!(
        "missing expectation {}; run with REDOX_BLESS=1 to create it",
        expected_path.display()
      )
    });

    assert_eq!(
      actual,
      expected,
      "golden mismatch for {}\n\nrun with REDOX_BLESS=1 to update",
      sdx.display()
    );
  }

  assert!(
    blessed.is_empty(),
    "blessed {} expectation(s); review the diff and re-run without REDOX_BLESS",
    blessed.len()
  );
}
