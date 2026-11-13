//! The manifest facts a crates.io publish depends on.
//!
//! `scripts/validate-crate-packaging.sh` is the authority on whether this
//! workspace can be published: it builds every archive and compiles it. But it
//! runs in the release workflow, so a manifest broken on a Tuesday is not found
//! until someone cuts a release. This test puts the cheap half of those rules —
//! the ones readable straight off the manifests, needing no cargo and no
//! network — into the ordinary gate.
//!
//! The rule it exists for is `include`. Every crate under `crates/` is written
//! in Voltaic: the `.sdx` under its `sdx/` is the source and the `.rs` under its
//! `src/` is generated and `.gitignore`d. `cargo package` enumerates files from
//! git, so **without an `include` key a published crate contains no Rust at
//! all** — and it packages and uploads perfectly cleanly. Measured before the
//! keys existed: `cargo package -p voltaic-ast --list` emitted `Cargo.toml`,
//! `moon.yml` and `sdx/*.sdx`, and nothing else.
//!
//! The shell script catches that by reading the built archive, which is the
//! authoritative check and the one that cannot be fooled. This file catches it
//! by reading the key, which is the one that reports it in seconds on the
//! commit that caused it. Two checks on one key is deliberate here, and it is
//! the only rule that is checked twice.
//!
//! `docs/RELEASING.md` is the prose version of all of this.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The crates that deliberately stay off crates.io, and why.
///
/// `voltaic-bench`, `voltaic-fuzz` and `voltaic-seed` are internal: the gates
/// and the bootstrap are their only consumers. `voltaic-lsp` and `voltaic-mcp`
/// are binary-only — nixpkgs and `flake.nix` build them and editors run them as
/// processes, so nothing links them as libraries.
///
/// Listed rather than derived on purpose. A crate silently falling off
/// crates.io is the failure this guards, so the expected set has to be written
/// down somewhere a reviewer reads.
const UNPUBLISHED: &[&str] = &[
  "voltaic-bench",
  "voltaic-fuzz",
  "voltaic-lsp",
  "voltaic-mcp",
  "voltaic-seed",
];

/// Every directory under `crates/` that is a crate.
fn crate_names() -> Vec<String> {
  let mut names: Vec<String> = std::fs::read_dir(workspace_root().join("crates"))
    .expect("should read crates/")
    .filter_map(|entry| {
      let path = entry.expect("should read a directory entry").path();
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

fn manifest_of(crate_name: &str) -> String {
  let path = workspace_root()
    .join("crates")
    .join(crate_name)
    .join("Cargo.toml");
  std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("should read {}: {e}", path.display()))
}

/// The lines of one `[section]`, up to the next header.
///
/// Parsed by hand rather than with a TOML crate, for the reason
/// `moon_project_graph.rs` gives: the shapes this has to read are
/// `key = "..."`, `key = { ... }` and `key.workspace = true` under a known
/// header, and reaching for a parser to read Cargo.toml is a circularity worth
/// avoiding.
fn section<'a>(manifest: &'a str, header: &str) -> Vec<&'a str> {
  let mut lines = Vec::new();
  let mut inside = false;
  for line in manifest.lines() {
    let trimmed = line.trim();
    if trimmed.starts_with('[') {
      inside = trimmed == header;
      continue;
    }
    if inside && !trimmed.is_empty() && !trimmed.starts_with('#') {
      lines.push(trimmed);
    }
  }
  lines
}

/// The value of `key = "..."` in a section, unquoted.
fn string_value(lines: &[&str], key: &str) -> Option<String> {
  lines
    .iter()
    .find_map(|line| line.strip_prefix(key)?.trim().strip_prefix('='))
    .map(|rest| rest.trim().trim_matches('"').to_owned())
}

fn is_published(crate_name: &str) -> bool {
  !UNPUBLISHED.contains(&crate_name)
}

/// The dependency entry for `dep` in `section_header`, if there is one.
fn dependency_line(manifest: &str, section_header: &str, dep: &str) -> Option<String> {
  section(manifest, section_header)
    .into_iter()
    .find(|line| line.starts_with(&format!("{dep} ")) || line.starts_with(&format!("{dep}.")))
    .map(str::to_owned)
}

#[test]
fn every_crate_is_either_published_or_explicitly_not() {
  let mut wrong = Vec::new();
  for name in crate_names() {
    let manifest = manifest_of(&name);
    let declares_false = section(&manifest, "[package]")
      .iter()
      .any(|line| line.replace(' ', "") == "publish=false");
    if declares_false == is_published(&name) {
      wrong.push(format!(
        "{name}: publish = false is {}, but UNPUBLISHED in this test says it should be {}",
        declares_false,
        !is_published(&name)
      ));
    }
  }
  assert!(
    wrong.is_empty(),
    "the manifests and this test disagree about what is published:\n  {}\n\
     Update UNPUBLISHED here and the table in docs/RELEASING.md together.",
    wrong.join("\n  ")
  );
}

#[test]
fn every_published_crate_has_a_description() {
  let mut missing = Vec::new();
  for name in crate_names().into_iter().filter(|n| is_published(n)) {
    let manifest = manifest_of(&name);
    let package = section(&manifest, "[package]");
    match string_value(&package, "description") {
      Some(description) if !description.is_empty() => {}
      _ => missing.push(name),
    }
  }
  assert!(
    missing.is_empty(),
    "crates.io rejects a publish with no description. Missing on: {}",
    missing.join(", ")
  );
}

#[test]
fn every_published_crate_includes_its_generated_rust() {
  let mut broken = Vec::new();
  for name in crate_names().into_iter().filter(|n| is_published(n)) {
    let manifest = manifest_of(&name);
    let include = section(&manifest, "[package]")
      .into_iter()
      .find(|line| line.starts_with("include"));
    match include {
      Some(line) if line.contains("\"src/**/*.rs\"") => {}
      Some(line) => broken.push(format!("{name}: include does not list src/**/*.rs: {line}")),
      None => broken.push(format!("{name}: no include key at all")),
    }
  }
  assert!(
    broken.is_empty(),
    "a published crate without include = [\"src/**/*.rs\", ...] ships no Rust:\n  {}\n\
     crates/<crate>/src/ is generated and .gitignore'd, and cargo packages from the\n\
     git file listing. See the header of this file and docs/RELEASING.md.",
    broken.join("\n  ")
  );
}

#[test]
fn a_crate_with_a_build_script_includes_it() {
  let mut broken = Vec::new();
  for name in crate_names().into_iter().filter(|n| is_published(n)) {
    let has_build_script = workspace_root()
      .join("crates")
      .join(&name)
      .join("build.rs")
      .is_file();
    if !has_build_script {
      continue;
    }
    let manifest = manifest_of(&name);
    let includes_it = section(&manifest, "[package]")
      .into_iter()
      .any(|line| line.starts_with("include") && line.contains("\"build.rs\""));
    if !includes_it {
      broken.push(name);
    }
  }
  assert!(
    broken.is_empty(),
    "these crates have a build.rs that their include key omits, so the published \
     crate cannot build: {}",
    broken.join(", ")
  );
}

#[test]
fn workspace_dependency_versions_match_the_workspace_version() {
  let root = std::fs::read_to_string(workspace_root().join("Cargo.toml"))
    .expect("should read the root Cargo.toml");

  let version = string_value(&section(&root, "[workspace.package]"), "version")
    .expect("[workspace.package] should set a version");

  let mut wrong = Vec::new();
  for line in section(&root, "[workspace.dependencies]") {
    let Some((name, _)) = line.split_once(' ') else {
      continue;
    };
    let declared = line
      .split_once("version = \"")
      .and_then(|(_, rest)| rest.split_once('"'))
      .map(|(v, _)| v);
    match declared {
      Some(v) if v == version => {}
      Some(v) => wrong.push(format!(
        "{name}: version = \"{v}\", workspace is \"{version}\""
      )),
      None => wrong.push(format!(
        "{name}: no version, so cargo publish will refuse it"
      )),
    }
  }

  // What this adds over cargo, measured rather than assumed. Setting one of
  // these to a version no crate has makes **cargo itself** refuse: "failed to
  // select a version for the requirement `voltaic-ast = ^0.9.9`", before any
  // test runs. So cargo already covers the drift case, and bumping
  // [workspace.package] while forgetting these cannot slip through.
  //
  // What cargo does not cover is a *missing* version. A path dependency with no
  // version resolves locally and builds fine; it fails only at `cargo publish`,
  // which is the worst moment to find out. That is the case this test catches
  // on its own, and it is why it is still here.
  assert!(
    wrong.is_empty(),
    "cargo cannot inherit a version inside [workspace.dependencies], so it is the one\n\
     place the version is written twice:\n  {}\n\
     docs/RELEASING.md step 1 bumps both.",
    wrong.join("\n  ")
  );
}

#[test]
fn published_crates_depend_only_on_published_crates() {
  let all: BTreeSet<String> = crate_names().into_iter().collect();
  let mut bad = Vec::new();
  for name in crate_names().into_iter().filter(|n| is_published(n)) {
    let manifest = manifest_of(&name);
    for dep in all.iter().filter(|d| !is_published(d)) {
      if dependency_line(&manifest, "[dependencies]", dep).is_some() {
        bad.push(format!("{name} -> {dep}"));
      }
    }
  }
  assert!(
    bad.is_empty(),
    "a published crate cannot have a runtime dependency on a publish = false crate — \
     crates.io could never resolve it: {}",
    bad.join(", ")
  );
}

#[test]
fn runtime_dependencies_on_workspace_crates_carry_a_version() {
  let all: BTreeSet<String> = crate_names().into_iter().collect();
  let mut bad = Vec::new();
  for name in crate_names().into_iter().filter(|n| is_published(n)) {
    let manifest = manifest_of(&name);
    for dep in &all {
      let Some(line) = dependency_line(&manifest, "[dependencies]", dep) else {
        continue;
      };
      // `.workspace = true` inherits path *and* version from
      // [workspace.dependencies], which the test above pins to the workspace
      // version. Anything else has to state a version itself.
      let inherits = line.replace(' ', "") == format!("{dep}.workspace=true");
      if !inherits && !line.contains("version") {
        bad.push(format!("{name}: {line}"));
      }
    }
  }
  assert!(
    bad.is_empty(),
    "cargo publish resolves a path dependency's version against crates.io, so a \
     runtime dependency with a path and no version cannot be published:\n  {}\n\
     Use `<crate>.workspace = true`.",
    bad.join("\n  ")
  );
}

#[test]
fn dev_dependencies_on_workspace_crates_are_path_only() {
  let all: BTreeSet<String> = crate_names().into_iter().collect();
  let mut bad = Vec::new();
  for name in crate_names() {
    let manifest = manifest_of(&name);
    for dep in &all {
      let Some(line) = dependency_line(&manifest, "[dev-dependencies]", dep) else {
        continue;
      };
      if line.contains("version") || line.replace(' ', "") == format!("{dep}.workspace=true") {
        bad.push(format!("{name}: {line}"));
      }
    }
  }
  assert!(
    bad.is_empty(),
    "this is the opposite rule to runtime dependencies, and it matters for the same \
     reason: cargo publish resolves a dev-dependency's version against crates.io even \
     when it also has a path, so a versioned dev-dep makes a crate's publish wait for a \
     sibling it does not need in order to build:\n  {}\n\
     Use `{{ path = \"../<crate>\" }}` with no version.",
    bad.join("\n  ")
  );
}
