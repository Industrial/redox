//! Bake a **working** `rustc` path into the crate that runs rustc.
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`) leaf 7 wrote this, for
//! `voltaic-cli`. Plan 50 (`docs/44 - 50 - MCP Server.md`) leaf 1 **moved it
//! here**, because `cargo::rustc-env` bakes a value into whichever crate
//! carries the build script and three crates now need the value:
//! `voltaic-cli`'s `check`, `voltaic-bench`'s IR comparison, and `voltaic-mcp`'s
//! `voltaic_check` tool. Before the move there were two copies of this file
//! differing **only in their doc comments** — `voltaic-cli`'s and
//! `voltaic-bench`'s — and a third was about to be written.
//!
//! Neither obvious source for a rustc path is reliable here, and each one
//! looks like the other's fix:
//!
//! - Resolving `rustc` from `PATH` **at run time** finds NixOS's `stub-ld`
//!   shim under `blaze test`, which cannot execute at all.
//! - Trusting `RUSTC` **at build time** finds blaze's own shim under
//!   `blaze build`, which refuses general rustc work with
//!   `BLAZE_SCRIPT_RUSTC does not name blaze's rustc`.
//!
//! So a candidate is not trusted, it is **verified**: each is run with
//! `--version` and the first that answers like a real rustc wins. See
//! `AGENTS.md`'s known tooling gotchas.
//!
//! A build script also sees something a test binary does not — the real
//! toolchain on `PATH` — which is why run-time probing alone is not enough.
//! `voltaic-bench` learned that the hard way: all four of its tests failed under
//! `blaze test` with *"no rustc answered `--version` like rustc"* while passing
//! under `cargo nextest`, because every candidate visible at run time was a
//! shim.

use std::path::PathBuf;
use std::process::Command;

fn main() {
  println!("cargo::rerun-if-env-changed=RUSTC");
  println!(
    "cargo::rustc-env=VOLTAIC_RUSTC={}",
    resolve_rustc().display()
  );
}

/// The first candidate that behaves like a real rustc.
fn resolve_rustc() -> PathBuf {
  // `PATH` first: in a build script it holds the toolchain, whereas `RUSTC`
  // may be a wrapper that only serves its own build graph.
  let from_path: Vec<PathBuf> = std::env::var_os("PATH")
    .map(|paths| {
      std::env::split_paths(&paths)
        .map(|d| d.join("rustc"))
        .collect()
    })
    .unwrap_or_default();

  let from_env = std::env::var_os("RUSTC").map(PathBuf::from);

  for candidate in from_path.into_iter().chain(from_env) {
    if is_real_rustc(&candidate) {
      return candidate;
    }
  }
  // Last resort: let the run-time probe report it, which it does by name.
  PathBuf::from("rustc")
}

/// Whether `candidate` answers `--version` like rustc itself.
///
/// A shim that proxies for a build graph fails here — either by exiting
/// non-zero or by printing something that is not a rustc version banner.
fn is_real_rustc(candidate: &PathBuf) -> bool {
  if !candidate.is_file() {
    return false;
  }
  Command::new(candidate)
    .arg("--version")
    .output()
    .is_ok_and(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).starts_with("rustc "))
}
