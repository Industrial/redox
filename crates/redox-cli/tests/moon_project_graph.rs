//! Each crate's `moon.yml` against its `Cargo.toml`.
//!
//! The build is per-crate: `.moon/tasks/tag-crate.yml` defines `check`,
//! `build`, `test`, their cargo twins and `lint` once, and every crate
//! inherits them through the `crate` tag. moon then caches each crate's
//! verdict and skips the crates whose inputs have not changed.
//!
//! That caching is only safe if a crate's inputs include the sources of the
//! crates it depends on. moon 2.5.0's task `deps` do **not** give this: they
//! set the order tasks run in, not the hash they are cached under. Measured by
//! appending a line to `crates/redox-diag/src/lib.rs` — which every other
//! crate depends on — and watching `moon run :check` re-run redox-diag alone
//! and report the other ten cached. A breaking change there would have been
//! reported green.
//!
//! So each `moon.yml` overrides the `dependency_sources` file group with its
//! transitive dependencies, and that list is a second copy of what
//! `Cargo.toml` already says. This test is what stops the copy drifting: add a
//! dependency and forget the `moon.yml`, and the crate's cached verdict starts
//! surviving changes that should invalidate it — a silent false green, which
//! is the worst kind.
//!
//! `dependsOn` is deliberately *not* declared: moon's rust toolchain reads the
//! edges out of Cargo.toml itself, so that one does not need repeating.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every directory under `crates/` that is a crate.
fn crate_names() -> Vec<String> {
  let mut names: Vec<String> = std::fs::read_dir(workspace_root().join("crates"))
    .expect("should read crates/")
    .filter_map(|e| {
      let path = e.expect("should read a directory entry").path();
      path
        .join("Cargo.toml")
        .is_file()
        .then(|| path.file_name()?.to_str().map(str::to_owned))
        .flatten()
    })
    .collect();
  names.sort();
  names
}

/// The crate names `manifest` depends on, from any dependency section.
///
/// Parsed by hand rather than with a TOML crate: this is a test, the shapes it
/// must read are `name = "..."`, `name = { ... }` and `name.workspace = true`
/// under a `[...dependencies]` header, and adding a dev-dependency on a TOML
/// parser to read Cargo.toml is a circularity worth avoiding.
fn direct_deps(manifest: &str, all: &BTreeSet<String>) -> BTreeSet<String> {
  let mut deps = BTreeSet::new();
  let mut in_deps = false;
  for line in manifest.lines() {
    let line = line.trim();
    if line.starts_with('[') {
      // `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, and
      // the `[target.'cfg(...)'.dependencies]` forms.
      in_deps = line.ends_with("dependencies]");
      continue;
    }
    if !in_deps || line.is_empty() || line.starts_with('#') {
      continue;
    }
    let Some((key, _)) = line.split_once('=') else {
      continue;
    };
    // `redox-lexer.workspace = true` names the crate before the dot.
    let name = key.trim().split('.').next().unwrap_or("").trim();
    if all.contains(name) {
      deps.insert(name.to_owned());
    }
  }
  deps
}

/// Everything `start` depends on, directly or through another crate.
fn closure(start: &str, direct: &BTreeMap<String, BTreeSet<String>>) -> BTreeSet<String> {
  let mut seen = BTreeSet::new();
  let mut stack: Vec<String> = direct[start].iter().cloned().collect();
  while let Some(next) = stack.pop() {
    if seen.insert(next.clone()) {
      stack.extend(direct[&next].iter().cloned());
    }
  }
  seen
}

/// The crate names appearing in a `moon.yml`'s `dependency_sources` group.
fn declared_sources(moon_yml: &str) -> BTreeSet<String> {
  let mut names = BTreeSet::new();
  for line in moon_yml.lines() {
    let line = line.trim();
    // `- '/crates/redox-lexer/src/**/*'` and the Cargo.toml line beside it.
    let Some(rest) = line.strip_prefix("- '/crates/") else {
      continue;
    };
    if let Some((name, _)) = rest.split_once('/') {
      names.insert(name.to_owned());
    }
  }
  names
}

fn manifests() -> (Vec<String>, BTreeMap<String, BTreeSet<String>>) {
  let names = crate_names();
  let all: BTreeSet<String> = names.iter().cloned().collect();
  let direct = names
    .iter()
    .map(|name| {
      let manifest = std::fs::read_to_string(
        workspace_root()
          .join("crates")
          .join(name)
          .join("Cargo.toml"),
      )
      .expect("should read a crate's Cargo.toml");
      (name.clone(), direct_deps(&manifest, &all))
    })
    .collect();
  (names, direct)
}

#[test]
fn every_crate_is_tagged_so_it_inherits_the_per_crate_tasks() {
  let names = crate_names();
  assert!(!names.is_empty(), "should find the workspace's crates");

  let untagged: Vec<String> = names
    .iter()
    .filter(|name| {
      let path = workspace_root().join("crates").join(name).join("moon.yml");
      match std::fs::read_to_string(&path) {
        Ok(text) => !text.contains("- 'crate'"),
        // No moon.yml at all is the same defect: the crate silently has no
        // check, no test and no lint, and the gate passes without it.
        Err(_) => true,
      }
    })
    .cloned()
    .collect();

  assert!(
    untagged.is_empty(),
    "these crates have no moon.yml tagged `crate`, so they inherit none of \
     the per-crate tasks and the gate skips them entirely:\n  {}",
    untagged.join("\n  ")
  );
}

#[test]
fn dependency_sources_match_the_transitive_cargo_dependencies() {
  let (names, direct) = manifests();

  let mut wrong = Vec::new();
  for name in &names {
    let expected = closure(name, &direct);
    let path = workspace_root().join("crates").join(name).join("moon.yml");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let declared = declared_sources(&text);

    // redox-cli's conformance tests read every crate's sources, so its group
    // is deliberately wider than its dependencies — `/crates/**/*.rs` covers
    // what the closure would, and more.
    if text.contains("- '/crates/**/*.rs'") {
      assert!(
        expected.is_subset(&declared) || declared.is_empty(),
        "{name}: declares the workspace-wide input but lost its own \
         dependency sources"
      );
      continue;
    }

    if declared != expected {
      let missing: Vec<&String> = expected.difference(&declared).collect();
      let extra: Vec<&String> = declared.difference(&expected).collect();
      wrong.push(format!("{name}: missing {missing:?}, unexpected {extra:?}"));
    }
  }

  assert!(
    wrong.is_empty(),
    "each crate's `dependency_sources` file group must list its TRANSITIVE \
     intra-workspace dependencies, or moon caches that crate's verdict \
     through a change it should have been invalidated by:\n  {}",
    wrong.join("\n  ")
  );
}
