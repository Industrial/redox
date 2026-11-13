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

/// Criterion: the platform package renders, and renders completely.
///
/// `design/rasterize.sh` treats any resvg warning as fatal, which is the
/// invariant that matters here: resvg exits 0 on an unmatched font or an
/// unparseable fill, having silently dropped what it could not draw. The first
/// version of that script sent stderr to `/dev/null` and produced three
/// banners with **zero** ink pixels while reporting success. So "it ran" is
/// not evidence, and "it ran without warning" is.
#[test]
fn the_platform_package_renders_without_a_single_warning() {
  let Some(dist) = scratch_render("renders") else {
    return;
  };
  for rel in [
    "web/favicon.ico",
    "web/favicon.svg",
    "web/apple-touch-icon.png",
    "web/icon-192.png",
    "web/icon-512.png",
    "web/maskable-512.png",
    "web/monochrome-512.png",
    "web/manifest.webmanifest",
    "ios/AppIcon.appiconset/Contents.json",
    "ios/AppIcon.appiconset/icon-1024.png",
    "macos/voltaic.icns",
    "windows/voltaic.ico",
    "windows/tiles/Square150x150.png",
    "linux/16x16/apps/voltaic.png",
    "linux/scalable/apps/voltaic.svg",
    "social/social-preview.png",
    "readme/header-light.png",
    "readme/header-dark.png",
  ] {
    assert!(
      dist.join(rel).is_file(),
      "design/rasterize.sh did not emit {rel}"
    );
  }
  let _ = std::fs::remove_dir_all(&dist);
}

/// Criterion: every ICO frame is the size it claims, read from the header.
///
/// An ICO is a container, and the usual defect is one 256px frame listed under
/// six sizes — which looks correct everywhere except the 16px slot it exists
/// for. So the directory is parsed rather than the file trusted.
#[test]
fn every_ico_frame_is_the_size_it_claims() {
  let Some(dist) = scratch_render("ico") else {
    return;
  };
  for (rel, want) in [
    ("web/favicon.ico", vec![16u32, 32, 48]),
    ("windows/voltaic.ico", vec![16, 24, 32, 48, 64, 256]),
  ] {
    let bytes = std::fs::read(dist.join(rel)).expect("an ico");
    assert_eq!(&bytes[0..4], &[0, 0, 1, 0], "{rel} is not an icon-type ICO");
    let count = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
    let mut sizes: Vec<u32> = (0..count)
      .map(|i| {
        let b = bytes[6 + i * 16];
        // The width field is one byte, so zero means 256.
        if b == 0 { 256 } else { u32::from(b) }
      })
      .collect();
    sizes.sort_unstable();
    let mut want = want;
    want.sort_unstable();
    assert_eq!(sizes, want, "{rel} frames");
  }
  let _ = std::fs::remove_dir_all(&dist);
}

/// Criterion: the manifest and the asset catalogue parse, and say what they
/// should.
///
/// Both are emitted by `jq`, so a syntax defect is unlikely — but the *content*
/// is this script's to get right, and a manifest missing its maskable purpose
/// is a manifest Android will crop.
#[test]
fn the_generated_json_declares_what_each_platform_needs() {
  let Some(dist) = scratch_render("json") else {
    return;
  };

  let manifest: serde_json::Value = serde_json::from_str(
    &std::fs::read_to_string(dist.join("web/manifest.webmanifest")).expect("the manifest"),
  )
  .expect("the manifest should be valid JSON");
  let purposes: Vec<&str> = manifest["icons"]
    .as_array()
    .expect("icons should be an array")
    .iter()
    .filter_map(|i| i["purpose"].as_str())
    .collect();
  assert!(
    purposes.contains(&"maskable"),
    "the manifest needs a maskable icon, or Android crops the mark"
  );
  assert!(
    purposes.contains(&"monochrome"),
    "the manifest needs a monochrome icon for themed icons"
  );

  let catalogue: serde_json::Value = serde_json::from_str(
    &std::fs::read_to_string(dist.join("ios/AppIcon.appiconset/Contents.json"))
      .expect("the catalogue"),
  )
  .expect("Contents.json should be valid JSON");
  let images = catalogue["images"]
    .as_array()
    .expect("images should be an array");
  assert_eq!(images.len(), 3, "iOS 18 wants default, dark and tinted");

  let _ = std::fs::remove_dir_all(&dist);
}

/// Render the package into a scratch directory outside the repository, or
/// `None` if the toolchain is absent — which is announced, not silent.
///
/// `std::env::temp_dir()` and a pid-tagged name, which is the convention
/// `support.rs` already uses: outside the tree, so nothing inherits this
/// repository's `rustfmt.toml`, and unique, so parallel tests cannot collide.
fn scratch_render(tag: &str) -> Option<std::path::PathBuf> {
  for tool in ["resvg", "oxipng", "icotool", "png2icns", "jq", "fc-list"] {
    let found = Command::new("sh")
      .arg("-c")
      .arg(format!("command -v {tool}"))
      .output()
      .map(|o| o.status.success())
      .unwrap_or(false);
    if !found {
      eprintln!("skipping: {tool} is not on PATH; run inside 'devenv shell'");
      return None;
    }
  }
  let dist = std::env::temp_dir().join(format!("voltaic-design-{tag}-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&dist);
  let out = Command::new("bash")
    .arg("design/rasterize.sh")
    .env("VOLTAIC_DESIGN_DIST", &dist)
    .current_dir(root())
    .output()
    .expect("design/rasterize.sh should be runnable");
  assert!(
    out.status.success(),
    "design/rasterize.sh failed\n{}{}",
    String::from_utf8_lossy(&out.stdout),
    String::from_utf8_lossy(&out.stderr)
  );
  Some(dist)
}
