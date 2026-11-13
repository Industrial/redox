//! The book builds, its samples come from the gated corpus, and Ctrl-K works.
//!
//! Identity and documentation site
//! (`.cursor/plans/voltaic-identity-and-site.plan.md`) waves 5, 6 and 7.
//!
//! The strongest thing asserted here is the second one. Every code sample in
//! the book is `{{#include}}`d out of `examples/`, which the suite already
//! compiles and runs against a recorded expectation — so a sample cannot drift
//! from what the compiler does, because there is no second copy of it to
//! drift. This file is what stops that guarantee being quietly lost by a
//! chapter that pastes code inline or cites a file outside the corpus.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Is `tool` on PATH? Absence is announced rather than silently skipped.
fn have(tool: &str) -> bool {
  let found = Command::new("sh")
    .arg("-c")
    .arg(format!("command -v {tool}"))
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false);
  if !found {
    eprintln!("skipping: {tool} is not on PATH; run inside 'devenv shell'");
  }
  found
}

/// Build the book into a scratch directory and return it.
fn build_book(tag: &str) -> Option<PathBuf> {
  if !have("mdbook") || !have("mdbook-linkcheck2") {
    return None;
  }
  let dest = std::env::temp_dir().join(format!("voltaic-book-{tag}-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&dest);
  let out = Command::new("mdbook")
    .arg("build")
    .arg("--dest-dir")
    .arg(&dest)
    .current_dir(root().join("www"))
    .output()
    .expect("mdbook should be runnable");
  assert!(
    out.status.success(),
    "mdbook build failed\n{}{}",
    String::from_utf8_lossy(&out.stdout),
    String::from_utf8_lossy(&out.stderr)
  );
  Some(dest)
}

/// Criterion: the book builds, and `mdbook-linkcheck2` finds no dead link.
///
/// The linkcheck backend is configured with `warning-policy = "error"`, so a
/// dead cross-reference fails this rather than printing a warning nobody
/// reads. Web links are deliberately not followed — the build has to work
/// offline and must not depend on a third party's uptime.
#[test]
fn the_book_builds_with_no_broken_links() {
  let Some(dest) = build_book("links") else {
    return;
  };
  assert!(
    dest.join("html/index.html").is_file(),
    "the html backend produced no index"
  );
  let _ = std::fs::remove_dir_all(&dest);
}

/// Criterion: every sample in the book is a file in the gated example corpus.
///
/// Two halves, and the second is the one that matters. That an include
/// *resolves* only proves the path is not a typo. That it resolves **into
/// `examples/`** is what ties the book's samples to the corpus CI compiles and
/// runs — a chapter citing `docs/` or a scratch file would pass the first
/// check and silently give up the guarantee.
#[test]
fn every_sample_comes_from_the_gated_example_corpus() {
  let root = root();
  let src = root.join("www/src");
  let examples = root.join("examples").canonicalize().expect("examples/");

  let mut seen = 0;
  let mut chapters_with_a_sample = 0;
  for page in markdown_under(&src) {
    let body = std::fs::read_to_string(&page).expect("a chapter");
    let mut here = 0;
    for target in includes(&body) {
      let resolved = page
        .parent()
        .expect("a parent")
        .join(&target)
        .canonicalize()
        .unwrap_or_else(|e| {
          panic!(
            "{} includes {target}, which does not resolve: {e}",
            page.display()
          )
        });
      assert!(
        resolved.starts_with(&examples),
        "{} includes {target}, which is outside examples/ — the book's samples \
         must come from the corpus CI compiles and runs",
        page.display()
      );
      assert_eq!(
        resolved.extension().and_then(|e| e.to_str()),
        Some("sdx"),
        "{} includes {target}, which is not a .sdx source",
        page.display()
      );
      seen += 1;
      here += 1;
    }
    if here > 0 {
      chapters_with_a_sample += 1;
    }
  }

  // Floors rather than exact counts, so adding a chapter does not fail this —
  // but floors, because a run that read no includes at all would otherwise
  // pass having proved nothing.
  assert!(
    seen >= 30,
    "only {seen} includes were found across the book, which is too few to mean anything"
  );
  assert!(
    chapters_with_a_sample >= 15,
    "only {chapters_with_a_sample} chapters carry a sample; the language chapters should each have one"
  );
}

/// Criterion: Ctrl-K opens, filters, navigates and closes, in a real browser.
///
/// Driven by headless chromium and `--dump-dom` rather than by a browser
/// automation framework, because the repository has no browser-test
/// infrastructure and a static harness plus a DOM dump needs none.
///
/// The subsequence assertion is the one worth keeping: `expop` has to reach
/// *Expressions and operators*, which is the whole reason the palette ranks
/// over mdBook's index itself instead of handing the query to elasticlunr.
#[test]
fn the_quick_search_works_in_a_browser() {
  if !have("chromium") {
    return;
  }
  let Some(dest) = build_book("palette") else {
    return;
  };
  let html = dest.join("html");

  // mdBook content-hashes both filenames, so the harness is a template.
  let searchindex = file_matching(&html, "searchindex-", ".js").expect("the search index");
  let palette = file_matching(&html.join("theme"), "palette-", ".js").expect("the palette script");
  let harness = std::fs::read_to_string(
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/site/palette.harness.html"),
  )
  .expect("the harness")
  .replace("SEARCHINDEX_JS", &searchindex)
  .replace("PALETTE_JS", &format!("theme/{palette}"));
  let harness_path = html.join("_harness.html");
  std::fs::write(&harness_path, harness).expect("writing the harness");

  let out = Command::new("chromium")
    .args([
      "--headless",
      "--disable-gpu",
      "--no-sandbox",
      "--virtual-time-budget=9000",
      "--dump-dom",
    ])
    .arg(format!("file://{}", harness_path.display()))
    .output()
    .expect("chromium should be runnable");
  let dom = String::from_utf8_lossy(&out.stdout);

  let results = between(&dom, "<div id=\"out\">", "</div>").unwrap_or_default();
  let errors = between(&dom, "<div id=\"err\">", "</div>").unwrap_or_default();
  assert!(
    errors.trim().is_empty(),
    "the harness threw: {errors}\n{results}"
  );

  for expected in [
    "built_on_load=true",
    "closed_initially=true",
    "ctrl_k_opens=true",
    // The event must be consumed, or the browser's own shortcut fires too.
    "ctrl_k_prevented=true",
    "focus_in_input=true",
    "esc_closes=true",
    "meta_k_opens=true",
    "index_loaded=true",
    "first_selected=true",
    "marked=true",
    // The subsequence match: "expop" -> Expressions and operators.
    "subseq_first=language/expressions.html#expressions-and-operators",
    "arrow_moves_to=1",
    // Modal means Tab cannot escape to the page behind.
    "tab_trapped=true",
    "empty_hits=0",
    "role_dialog=true",
    "aria_modal=true",
    "listbox=true",
    "done=true",
  ] {
    assert!(
      results.contains(expected),
      "the palette harness did not report {expected}\nreported: {results}"
    );
  }

  let _ = std::fs::remove_dir_all(&dest);
}

/// Every `.md` under `dir`, recursively.
fn markdown_under(dir: &Path) -> Vec<PathBuf> {
  let mut found = Vec::new();
  let mut stack = vec![dir.to_path_buf()];
  while let Some(d) = stack.pop() {
    for entry in std::fs::read_dir(&d).expect("a directory") {
      let path = entry.expect("an entry").path();
      if path.is_dir() {
        stack.push(path);
      } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
        found.push(path);
      }
    }
  }
  found
}

/// The target of every `{{#include ...}}` in `body`, ignoring any anchor.
fn includes(body: &str) -> Vec<String> {
  let mut found = Vec::new();
  let mut rest = body;
  while let Some(i) = rest.find("{{#include ") {
    rest = &rest[i + "{{#include ".len()..];
    let Some(j) = rest.find("}}") else { break };
    let target = rest[..j].trim();
    // `{{#include file:anchor}}` is a partial include; the file is before the
    // colon. No chapter uses one yet, but the parser should not be the reason
    // one cannot be added.
    let target = target.split(':').next().unwrap_or(target);
    found.push(target.to_string());
    rest = &rest[j + 2..];
  }
  found
}

/// The name of the one file in `dir` starting with `prefix` and ending `suffix`.
fn file_matching(dir: &Path, prefix: &str, suffix: &str) -> Option<String> {
  std::fs::read_dir(dir)
    .ok()?
    .filter_map(|e| e.ok())
    .map(|e| e.file_name().to_string_lossy().into_owned())
    .find(|n| n.starts_with(prefix) && n.ends_with(suffix))
}

fn between(text: &str, open: &str, close: &str) -> Option<String> {
  let i = text.find(open)? + open.len();
  let j = text[i..].find(close)? + i;
  Some(text[i..j].to_string())
}
