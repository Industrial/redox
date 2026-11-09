//! Bake a **working** `rustc` path into this crate, exactly as `redox-cli`'s
//! build script does.
//!
//! Plan 44 (`docs/36 - 44 - Zero-Cost Benchmarks.md`). The zero-cost
//! comparison compiles each half of a pair with real rustc and diffs the IR,
//! so it needs a rustc that runs. `src/ir/rustc_path.rs` probes for one at
//! **run** time, and under `blaze test` every candidate it can see is a shim:
//! `PATH` holds NixOS's `stub-ld`, and `RUSTC` holds blaze's own, which
//! refuses general work. So all four of this crate's tests failed with
//!
//!   no rustc answered `--version` like rustc
//!
//! under `blaze test -p redox-bench` while passing under `cargo nextest`,
//! which is the trap AGENTS.md records and the reason redox-cli has a build
//! script at all. A build script runs with the real toolchain on `PATH`, so
//! the verified path can be found there and baked in for the tests to use.
//!
//! The rule is AGENTS.md's: a candidate is not trusted, it is verified by
//! running `--version` and reading the banner.

use std::path::PathBuf;
use std::process::Command;

fn main() {
  println!("cargo::rerun-if-env-changed=RUSTC");
  println!("cargo::rustc-env=REDOX_RUSTC={}", resolve_rustc().display());
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
  // Last resort: let `rustc_path`'s own probe report the failure.
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
