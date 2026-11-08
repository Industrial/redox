//! A Redox crate builds and runs under plain Cargo.
//!
//! Plan 09 (`docs/14 - 09 - Cargo Integration.md`) leaf 4. Doc 02 §6.6 says
//! the cargo question *"must be closed before Phase 2, because it determines
//! what a Redox project even looks like"*, so the proof is a project: the
//! crate under `examples/cargo-project/` is built by real Cargo and executed.
//!
//! Not a unit test of `compile_dir`. The claim is about Cargo — that a build
//! script plus `include!` is enough, with no new tooling — and only Cargo can
//! settle it. This test is also how plan 09 discovered that Redox cannot yet
//! produce a *reachable* item inside a module (see `src/main.rs`'s own note),
//! which no unit test would have surfaced.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

fn project() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/cargo-project")
}

/// Run `cargo` in the example project, from a clean environment.
///
/// The project detaches itself from this workspace with an empty
/// `[workspace]` table, so it builds the way a user's crate would — and the
/// *environment* has to match that claim too.
///
/// Every `CARGO_*`/`RUST*` variable the outer build set is removed. Inheriting
/// them is not a theoretical hazard: under `blaze test` the inherited `RUSTC`
/// and `CARGO` point into `~/.blaze/toolchains/`, whose binaries NixOS cannot
/// execute, and the nested build failed with **empty stderr** — the least
/// debuggable failure there is. `RUSTC` is then set back to the one
/// `build.rs` verified, so the child uses a compiler known to work.
fn cargo(args: &[&str]) -> std::process::Output {
  let cargo = support::cargo_path().expect("caller should check skip_without_cargo");
  let mut command = Command::new(cargo);
  command.args(args).current_dir(project());

  for (key, _) in std::env::vars_os() {
    let name = key.to_string_lossy();
    if name.starts_with("CARGO") || name.starts_with("RUST") {
      command.env_remove(&key);
    }
  }
  command.env("RUSTC", support::rustc());

  command
    .output()
    .expect("should run cargo in the example project")
}

/// Criterion 4: the project builds and runs under plain Cargo.
#[test]
fn the_example_project_builds_and_runs() {
  if support::skip_without_rustc("the_example_project_builds_and_runs")
    || support::skip_without_cargo("the_example_project_builds_and_runs")
  {
    return;
  }
  let out = cargo(&["run", "--quiet"]);
  assert!(
    out.status.success(),
    "`cargo run` should succeed in examples/cargo-project:\n{}",
    String::from_utf8_lossy(&out.stderr)
  );
  assert_eq!(
    String::from_utf8_lossy(&out.stdout),
    "hello, world\n42\n",
    "the project's own output"
  );
}

/// Criterion 1: the build script wrote `.rs` into `OUT_DIR`, with a map
/// beside it.
///
/// `.rs` because that is what rustc accepts; the `.xrs` remains the
/// human-facing artifact `redox build` writes (doc 02 §6.6).
#[test]
fn the_build_script_writes_rust_and_a_map_into_out_dir() {
  if support::skip_without_rustc("the_build_script_writes_rust_and_a_map_into_out_dir")
    || support::skip_without_cargo("the_build_script_writes_rust_and_a_map_into_out_dir")
  {
    return;
  }
  assert!(cargo(&["build", "--quiet"]).status.success());

  let build_dir = project().join("target/debug/build");
  let mut generated = Vec::new();
  for entry in std::fs::read_dir(&build_dir).expect("target/debug/build should exist") {
    let out = entry.expect("an entry").path().join("out");
    let rs = out.join("greeting.rs");
    if rs.is_file() {
      generated.push(rs);
    }
  }
  assert!(
    !generated.is_empty(),
    "the build script should have written greeting.rs under {}",
    build_dir.display()
  );

  for rs in &generated {
    let rust = std::fs::read_to_string(rs).expect("should read the generated Rust");
    assert!(
      rust.contains("fn greet"),
      "the generated file should hold the lowered function:\n{rust}"
    );

    let map = rs.with_file_name("greeting.rs.map");
    assert!(
      map.is_file(),
      "a map should sit beside the generated Rust at {}",
      map.display()
    );
    let parsed =
      redox_diag::parse_map(&std::fs::read_to_string(&map).expect("should read the map"))
        .expect("the map should parse");
    assert!(
      parsed.source_path.ends_with("greeting.sdx"),
      "the map should name the .sdx it came from, got {}",
      parsed.source_path
    );
  }
}

/// Criterion 5: editing a `.sdx` rebuilds.
///
/// Asserted through the directives rather than by editing a file and timing a
/// rebuild: the latter is what Cargo guarantees given correct directives, and
/// a timing test would be slow and flaky while proving less.
#[test]
fn the_build_script_asks_cargo_to_watch_the_sdx() {
  let compiled = redox_driver::compile_dir(
    &project().join("src"),
    &std::env::temp_dir().join(format!("redox-rerun-{}", std::process::id())),
  )
  .expect("the example project's sources should compile");

  let directives = compiled.rerun_directives();
  assert!(
    directives.iter().any(|d| d.ends_with("greeting.sdx")),
    "Cargo must be told to watch greeting.sdx, or editing it would not \
     rebuild and the user would run stale code: {directives:?}"
  );
  assert!(
    directives
      .iter()
      .all(|d| d.starts_with("cargo::rerun-if-changed=")),
    "{directives:?}"
  );
}

/// Criterion 6, as corrected by §2.2: `redox check` reports a type error in
/// the project's `.sdx` against the `.sdx`.
///
/// Plain `cargo build` does **not** — it reports against `OUT_DIR`, because
/// Cargo invokes rustc directly and nothing consults the map. That is the
/// honest limit of doc 02 §6.6 option (a), measured in plan 09 §2.2 and
/// disclosed rather than designed around. The map is written beside the
/// generated `.rs` so the information exists; `redox check` is what uses it.
#[test]
fn redox_check_reports_a_project_error_against_the_sdx() {
  if support::skip_without_rustc("redox_check_reports_a_project_error_against_the_sdx") {
    return;
  }
  let dir = std::env::temp_dir().join(format!("redox-proj-check-{}", std::process::id()));
  std::fs::create_dir_all(&dir).expect("should create");
  let sdx = dir.join("greeting.sdx");
  std::fs::write(
    &sdx,
    "# A deliberate type error.\ntwice = (n: i32): i32 ->\n  n * \"two\"\n",
  )
  .expect("should write");

  let out = Command::new(env!("CARGO_BIN_EXE_redox"))
    .args(["check"])
    .arg(&sdx)
    .arg("--plain")
    .output()
    .expect("should run redox check");
  let stderr = String::from_utf8_lossy(&out.stderr);

  assert!(!out.status.success(), "a type error should fail the check");
  assert!(
    stderr.contains("greeting.sdx:3:"),
    "should point into the .sdx, not OUT_DIR:\n{stderr}"
  );
  assert!(
    !stderr.contains(".rs:"),
    "the user should not be shown a generated file:\n{stderr}"
  );
  std::fs::remove_dir_all(&dir).ok();
}

/// A parse error in a project's source comes back as diagnostics the build
/// script can render, not as a panic or a bare string.
#[test]
fn a_broken_source_returns_renderable_diagnostics() {
  let dir = std::env::temp_dir().join(format!("redox-proj-bad-{}", std::process::id()));
  std::fs::create_dir_all(&dir).expect("should create");
  std::fs::write(dir.join("bad.sdx"), "main = ->\n  print 1 +\n").expect("should write");

  let errors = redox_driver::compile_dir(&dir, &dir.join("out"))
    .expect_err("a parse error should be reported");
  assert!(!errors.is_empty());
  let rendered = redox_diag::render(&errors, true);
  assert!(rendered.contains("error[RDX"), "{rendered}");
  assert!(rendered.contains("bad.sdx"), "{rendered}");
  std::fs::remove_dir_all(&dir).ok();
}
