//! `crates/redox-ast` is written in Redox — plan 68.
//!
//! `conformance/redox-ast/**/*.sdx` is the **source**; the `.rs` under
//! `crates/redox-ast/src/` is generated from it and committed, and cargo
//! compiles that exactly as it compiled the hand-written Rust. So the whole
//! workspace and all of its tests run against Redox-generated types, which is
//! the assertion plan 48's port could not make: a module lifted out of a crate
//! cannot resolve `crate::Stmt`, so its oracle had to be `syn::parse_file`
//! rather than rustc.
//!
//! # Why a test and not a `build.rs`
//!
//! Plan 68 §1.2. `compile_dir` lives in `redox-driver`, which depends on
//! `redox-ast` — a build script there is a dependency cycle cargo rejects.
//! Plan 09 built that path for a *user's* crate, where no cycle exists, and
//! `examples/cargo-project/` is its witness.
//!
//! Regeneration being a test rather than a build step catches both failure
//! directions with one assertion: a `.sdx` edited without regenerating, and a
//! hand edit to the generated Rust.
//!
//! ```bash
//! REDOX_BLESS=1 cargo test -p redox-cli --test ast_port
//! ```
//!
//! # The rustfmt step, and why it is not optional
//!
//! Redox's output targets rustfmt's **defaults** — four-space indentation —
//! and this repository's `rustfmt.toml` sets **two**, because that governs
//! Redox's own source. A generated file committed into `src/` is graded by the
//! repository's config like every other file there, so the pipeline is
//! `redox build` *and then* rustfmt with the repo's config. rustfmt is
//! deterministic, so the result is still byte-exact.
//!
//! `support::rustfmt` deliberately formats **outside** the tree to grade the
//! emitter against rustfmt's defaults (plan 04's probe is why). This needs the
//! opposite, so it runs rustfmt in place.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `.sdx` of the port, with the `.rs` it generates.
///
/// The mirror is one file for one file and the same relative path, which is
/// what keeps `docs/06`'s architecture rules applying: `one_construct_per_file`
/// and `directory_structure` scan `crates/**/src/**/*.rs`, so they grade the
/// **generated** tree — correctly, since the rules are about the Rust module
/// tree — and the `.sdx` tree inherits them through the mirror.
fn pairs() -> Vec<(PathBuf, PathBuf)> {
  let root = workspace_root();
  let from = root.join("conformance/redox-ast");
  let mut found = Vec::new();
  collect(&from, &mut found);
  found.sort();
  found
    .into_iter()
    .map(|sdx| {
      let rel = sdx
        .strip_prefix(&from)
        .expect("under conformance/redox-ast")
        .with_extension("rs");
      (sdx.clone(), root.join("crates/redox-ast/src").join(rel))
    })
    .collect()
}

fn collect(dir: &Path, into: &mut Vec<PathBuf>) {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if path.is_dir() {
      collect(&path, into);
    } else if path.extension().is_some_and(|e| e == "sdx") {
      into.push(path);
    }
  }
}

/// `redox build` on one file, then rustfmt with the repository's config.
fn generate(sdx: &Path) -> String {
  let out = Command::new(env!("CARGO_BIN_EXE_redox"))
    .arg("build")
    .arg(sdx)
    .output()
    .expect("should run redox build");
  assert!(
    out.status.success(),
    "{} should transpile:\n{}",
    sdx.display(),
    String::from_utf8_lossy(&out.stdout)
  );
  let xrs = sdx.with_extension("xrs");
  let rust = std::fs::read_to_string(&xrs).expect("should read the generated .xrs");
  support::rustfmt_in_place(&rust)
}

/// Whether the staged generation compiles, checked **before** it replaces
/// what is committed.
///
/// `cargo check -p redox-ast` over a swapped-in `src/`, with the real one
/// moved aside and restored whatever the answer is. A `Drop` guard would be
/// tidier; the explicit restore is here because the failure mode this
/// protects against is precisely a `src/` left in the wrong state.
fn staged_crate_compiles(root: &Path, staging: &Path) -> bool {
  let live = root.join("crates/redox-ast/src");
  let aside = root.join("target/redox-port-live");
  std::fs::remove_dir_all(&aside).ok();
  if std::fs::rename(&live, &aside).is_err() {
    return false;
  }
  let swapped = std::fs::rename(staging, &live).is_ok();
  let ok = swapped
    && Command::new("cargo")
      .args(["check", "-p", "redox-ast", "--quiet"])
      .current_dir(root)
      .status()
      .is_ok_and(|s| s.success());
  if swapped {
    std::fs::rename(&live, staging).ok();
  }
  std::fs::rename(&aside, &live).ok();
  ok
}

/// Criterion 1 and 4's other half: the trees are the same shape.
///
/// Both directions. A `.sdx` with no `.rs` means the port was never
/// regenerated; a `.rs` with no `.sdx` means a file was added to the crate in
/// Rust, which is the failure mode this row exists to prevent — the crate is
/// written in Redox now, and a hand-written module would compile and silently
/// not be.
#[test]
fn the_sdx_tree_mirrors_the_rust_tree() {
  let root = workspace_root();
  let pairs = pairs();
  assert!(
    !pairs.is_empty(),
    "conformance/redox-ast/ should hold the port's source"
  );

  let mut missing = Vec::new();
  for (sdx, rs) in &pairs {
    if !rs.exists() {
      missing.push(format!(
        "  {} has no generated {}",
        sdx.display(),
        rs.display()
      ));
    }
  }

  let mut unported = Vec::new();
  let src = root.join("crates/redox-ast/src");
  let mut rust = Vec::new();
  collect_rs(&src, &mut rust);
  for path in rust {
    let rel = path
      .strip_prefix(&src)
      .expect("under src")
      .with_extension("sdx");
    if !root.join("conformance/redox-ast").join(&rel).exists() {
      unported.push(format!("  {} has no .sdx source", path.display()));
    }
  }

  assert!(
    missing.is_empty() && unported.is_empty(),
    "the port's trees have drifted:\n{}\n{}",
    missing.join("\n"),
    unported.join("\n")
  );
}

fn collect_rs(dir: &Path, into: &mut Vec<PathBuf>) {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if path.is_dir() {
      collect_rs(&path, into);
    } else if path.extension().is_some_and(|e| e == "rs") {
      into.push(path);
    }
  }
}

/// Criterion 4: regenerating reproduces what is committed.
#[test]
fn the_port_regenerates_the_committed_rust() {
  if support::skip_without_rustc("the_port_regenerates_the_committed_rust") {
    return;
  }
  if !support::rustfmt_available() {
    eprintln!("skipping the_port_regenerates_the_committed_rust: no rustfmt");
    return;
  }

  let blessing = std::env::var("REDOX_BLESS").is_ok();
  let mut stale = Vec::new();
  let mut fresh = Vec::new();
  for (sdx, rs) in pairs() {
    let generated = generate(&sdx);
    if blessing {
      fresh.push((rs, generated));
      continue;
    }
    let committed = std::fs::read_to_string(&rs).unwrap_or_default();
    if committed != generated {
      stale.push(rs.display().to_string());
    }
  }

  // **Nothing is written until the whole generation compiles.**
  //
  // Found the hard way: blessing a generation with three rustc errors in it
  // wrote broken Rust into `crates/redox-ast/src/`, and `redox-ast` is the
  // dependency root — so `cargo run -p redox-cli` could no longer build the
  // compiler that would regenerate it. `git checkout -- crates/redox-ast/src/`
  // is the recovery, and it works only because the generated Rust is
  // **committed**: doc 02 §4.8's *"commit the generated `.xrs` so the
  // bootstrap never breaks"* earning its keep.
  //
  // So the files go to a staging directory, `cargo check -p redox-ast` runs
  // against them, and only then do they replace what is committed. A
  // generation that does not compile is reported and discarded, which leaves
  // the bootstrap exactly as it was.
  if blessing {
    let root = workspace_root();
    let staging = root.join("target/redox-port-staging");
    std::fs::remove_dir_all(&staging).ok();
    for (rs, text) in &fresh {
      let rel = rs
        .strip_prefix(root.join("crates/redox-ast/src"))
        .expect("under redox-ast/src");
      let at = staging.join(rel);
      std::fs::create_dir_all(at.parent().expect("a parent")).expect("should create it");
      std::fs::write(&at, text).expect("should stage the generated Rust");
    }
    assert!(
      staged_crate_compiles(&root, &staging),
      "the generated crate does not compile, so nothing was written — \
       `crates/redox-ast/src/` is untouched and the bootstrap still works. \
       The staged output is under target/redox-port-staging/."
    );
    for (rs, text) in &fresh {
      if let Some(parent) = rs.parent() {
        std::fs::create_dir_all(parent).expect("should create the module directory");
      }
      std::fs::write(rs, text).expect("should write the generated Rust");
    }
  }

  assert!(
    blessing || stale.is_empty(),
    "these generated files no longer match their .sdx source:\n  {}\n\
     Either the .sdx was edited without regenerating, or the .rs was edited by \
     hand — the .sdx is the source. Regenerate with:\n  \
     REDOX_BLESS=1 cargo test -p redox-cli --test ast_port",
    stale.join("\n  ")
  );
  assert!(
    !blessing,
    "blessed the port — review the diff rather than absorbing it"
  );
}

/// Criterion 7: no escape hatch anywhere in the port.
///
/// Doc 02 §4.8's criterion is that the compiler's source is expressible
/// **without** inline Rust, so one `rust` block would make the whole claim
/// vacuous. Plan 48 asserts the same thing for `clause.sdx` by looking at the
/// generated output; this looks at the source, which is stricter — plan 64
/// leaf 4 gave the hatch an expression position, and `rust("` no longer
/// appears in everything it generates.
#[test]
fn the_port_uses_no_escape_hatch() {
  let mut offenders = Vec::new();
  for (sdx, _) in pairs() {
    let text = std::fs::read_to_string(&sdx).expect("should read the port source");
    for (n, line) in text.lines().enumerate() {
      let code = line.split("--").next().unwrap_or("");
      if code.split_whitespace().any(|w| w == "rust") {
        offenders.push(format!("  {}:{}", sdx.display(), n + 1));
      }
    }
  }
  assert!(
    offenders.is_empty(),
    "the port reaches for the inline-Rust escape hatch:\n{}\n\
     Doc 02 §4.8's criterion is that the source is expressible *without* it.",
    offenders.join("\n")
  );
}

/// Criterion 5, and plan 68 leaf 4: every doc comment is still a doc comment.
///
/// **The only property here no other test could notice.** A doc comment in
/// Redox is decided **per line** — `-- |` is one and `--` is an ordinary
/// comment — so writing `-- |` once and `--` after produces valid Redox whose
/// output is `///` and then `//`. The text survives, so nothing that checks
/// for comment *loss* sees anything wrong, and the line is no longer
/// documentation. `redox-ast` is roughly 60% doc comment by line, and most of
/// them are multi-line.
///
/// Asserted as an equality between the two trees rather than against a
/// recorded figure: a count written down here would be one more number to
/// chase, and the `.sdx` is the source of truth for how many there should be.
#[test]
fn every_doc_comment_reaches_the_generated_rust() {
  let mut offenders = Vec::new();
  for (sdx, rs) in pairs() {
    let source = std::fs::read_to_string(&sdx).expect("should read the port source");
    let generated = std::fs::read_to_string(&rs).unwrap_or_default();

    let outer = source
      .lines()
      .filter(|l| l.trim_start().starts_with("-- |"))
      .count();
    let inner = source
      .lines()
      .filter(|l| l.trim_start().starts_with("-- !"))
      .count();
    let got_outer = generated
      .lines()
      .filter(|l| l.trim_start().starts_with("///"))
      .count();
    let got_inner = generated
      .lines()
      .filter(|l| l.trim_start().starts_with("//!"))
      .count();

    if outer != got_outer || inner != got_inner {
      offenders.push(format!(
        "  {}: {outer} `-- |` and {inner} `-- !` became {got_outer} `///` and {got_inner} `//!`",
        sdx.display()
      ));
    }
  }
  assert!(
    offenders.is_empty(),
    "doc comments did not survive the port:\n{}\n\
     A `--` continuation line is an *ordinary* comment: its text reaches the \
     output and its `///` does not, which no other test in this gate notices.",
    offenders.join("\n")
  );
}
