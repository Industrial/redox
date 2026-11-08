//! Bake the toolchain's `rustc` path into the test binaries.
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`), leaf 7. The end-to-end test
//! must invoke real rustc on the generated `.xrs`, and resolving `rustc` from
//! `PATH` at test time is not reliable here: under `blaze test` the test
//! process sees NixOS's `stub-ld` shim instead of the toolchain's rustc, and
//! fails with "Could not start dynamically linked executable: rustc".
//!
//! A build script runs with the build environment intact, so resolving the
//! path here and passing it through as a compile-time env var is stable
//! regardless of what the test process inherits. See `AGENTS.md`'s known
//! tooling gotchas.

use std::path::PathBuf;

fn main() {
  println!("cargo::rerun-if-env-changed=RUSTC");
  println!("cargo::rustc-env=REDOX_RUSTC={}", resolve_rustc().display());
}

/// Resolve rustc as absolutely as the build environment allows.
fn resolve_rustc() -> PathBuf {
  // Cargo sets `RUSTC` for build scripts, and it is already absolute.
  if let Some(from_env) = std::env::var_os("RUSTC") {
    let path = PathBuf::from(from_env);
    if path.is_absolute() {
      return path;
    }
  }
  // Otherwise walk `PATH`, which a build script sees correctly even when the
  // eventual test process does not.
  if let Some(paths) = std::env::var_os("PATH") {
    for dir in std::env::split_paths(&paths) {
      let candidate = dir.join("rustc");
      if candidate.is_file() {
        return candidate;
      }
    }
  }
  // Last resort: let the test's own executability probe report it.
  PathBuf::from("rustc")
}
