//! The identity assets are generated, and this is what makes that safe.
//!
//! Identity and documentation site
//! (`.cursor/plans/voltaic-identity-and-site.plan.md`) wave 2.
//!
//! `design/masters/*.svg` and `design/tokens/tokens.{css,toml}` are emitted by
//! `design/build.sh` from exactly two sources — `design/geometry/pile.json`
//! for the geometry and `design/tokens/tokens.json` for the palette — and then
//! committed, because their consumers live outside this build. A tracked
//! generated file drifts the moment someone edits it directly, so the gate
//! below regenerates into a temporary directory and diffs.
//!
//! That is the same discipline `seed/` already uses: the artifact is committed
//! for the consumers that need it, and a test proves it is still what its
//! source produces.

use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Criterion: every committed asset is what `design/build.sh` emits today.
///
/// The single strongest assertion available, because it covers the masters,
/// the favicon, `tokens.css` and `tokens.toml` in one pass and cannot go stale
/// as outputs are added — the script's own output list is the list.
#[test]
fn every_generated_asset_matches_its_sources() {
  let out = Command::new("bash")
    .arg("design/build.sh")
    .arg("--check")
    .current_dir(root())
    .output()
    .expect("design/build.sh should be runnable");
  assert!(
    out.status.success(),
    "design/build.sh --check failed; run ./design/build.sh and review the diff\n{}{}",
    String::from_utf8_lossy(&out.stdout),
    String::from_utf8_lossy(&out.stderr)
  );
}

/// Criterion: no hex literal exists outside `tokens.json`.
///
/// The palette has one definition. A colour that appears in a master but not
/// in the token file means someone hand-edited a generated asset, or the
/// generator grew a hardcoded value — both of which this catches before the
/// editor themes and the book's stylesheet inherit the drift.
#[test]
fn every_hex_in_a_generated_asset_comes_from_the_token_file() {
  let root = root();
  let tokens = std::fs::read_to_string(root.join("design/tokens/tokens.json"))
    .expect("design/tokens/tokens.json");
  let tokens = tokens.to_ascii_uppercase();

  let mut checked = 0;
  for entry in std::fs::read_dir(root.join("design/masters")).expect("design/masters") {
    let path = entry.expect("a dir entry").path();
    if path.extension().and_then(|e| e.to_str()) != Some("svg") {
      continue;
    }
    let body = std::fs::read_to_string(&path).expect("a master");
    for hex in hexes(&body) {
      assert!(
        tokens.contains(&hex),
        "{} carries {hex}, which is not in design/tokens/tokens.json",
        path.display()
      );
      checked += 1;
    }
  }
  // The mono cut uses `currentColor` and contributes none, so a floor rather
  // than an exact count — but a floor, because a run that read zero hexes
  // would pass vacuously and prove nothing.
  assert!(
    checked >= 20,
    "only {checked} hex literals were read across the masters, which is too \
     few to mean anything"
  );
}

/// Criterion: both optical cuts exist, are distinct, and share one viewBox.
///
/// The two-cut system is the part of this identity most easily lost — a future
/// edit that "simplifies" the generator to one drawing would silently give the
/// 16px favicon the five-plate cut, which smears. Asserting the cuts differ is
/// what makes that a failure rather than a regression nobody sees.
#[test]
fn the_two_optical_cuts_are_distinct_and_share_a_viewbox() {
  let root = root();
  let display = std::fs::read_to_string(root.join("design/masters/pile-display-light.svg"))
    .expect("the display cut");
  let small = std::fs::read_to_string(root.join("design/masters/pile-small-light.svg"))
    .expect("the small cut");

  assert_ne!(
    display, small,
    "the display and small cuts are byte-identical, so the optical-size system is gone"
  );
  for (name, body) in [("display", &display), ("small", &small)] {
    assert!(
      body.contains(r#"viewBox="0 0 64 64""#),
      "the {name} cut must be on a 0 0 64 64 viewBox"
    );
  }
  // Five plates plus a terminal against three plus a terminal.
  assert_eq!(
    display.matches("<rect").count(),
    6,
    "the display cut is six rects"
  );
  assert_eq!(
    small.matches("<rect").count(),
    4,
    "the small cut is four rects"
  );
}

/// Criterion: the favicon follows the browser's theme on its own.
///
/// A single favicon that adapts is why the masters are literal and this one
/// file is not — see `design/build.sh`'s header for the resvg measurement
/// behind that split.
#[test]
fn the_favicon_carries_a_colour_scheme_query() {
  let body =
    std::fs::read_to_string(root().join("design/masters/favicon.svg")).expect("the favicon");
  assert!(
    body.contains("prefers-color-scheme: dark"),
    "favicon.svg must adapt to the browser theme by itself"
  );
  // The small cut, because a tab icon is 16px.
  assert_eq!(
    body.matches("<rect").count(),
    4,
    "the favicon uses the small cut"
  );
}

/// Every `#RRGGBB` in `text`, upper-cased.
fn hexes(text: &str) -> Vec<String> {
  let bytes: Vec<char> = text.chars().collect();
  let mut found = Vec::new();
  for (i, c) in bytes.iter().enumerate() {
    if *c != '#' || i + 6 >= bytes.len() {
      continue;
    }
    let candidate: String = bytes[i + 1..i + 7].iter().collect();
    if candidate.len() == 6 && candidate.chars().all(|c| c.is_ascii_hexdigit()) {
      found.push(format!("#{}", candidate.to_ascii_uppercase()));
    }
  }
  found
}
