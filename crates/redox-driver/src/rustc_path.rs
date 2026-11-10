/// Whether a candidate answers `--version` the way rustc itself does.
pub(crate) mod is_real_rustc;

use crate::rustc_path::is_real_rustc::is_real_rustc;
use std::path::PathBuf;

/// A rustc that behaves like rustc, resolved at run time.
///
/// Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) leaf 5 wrote this inside
/// `redox-cli`'s `check_command`, on the stated grounds that *"the day `fmt`
/// or `watch` also needs a verified toolchain path is the day it earns a home
/// further up"*. Plan 50 (`docs/44 - 50 - MCP Server.md`) leaf 1 is that day,
/// with three callers rather than two: `redox check`, `redox-bench`'s IR
/// comparison, and `redox-mcp`'s `redox_check`.
///
/// # The order, and why both halves of it are load-bearing
///
/// `PATH` first, then the `REDOX_RUSTC` that `build.rs` baked in. The two
/// crates this consolidates **disagreed** about that order, and each had a
/// real reason:
///
/// - `redox-cli` put `PATH` first, because a shipped binary must not depend on
///   the rustc that happened to be installed when it was compiled.
/// - `redox-bench` put the baked value first, because under `blaze test` every
///   candidate visible at run time is a shim and all four of its tests failed
///   with *"no rustc answered `--version` like rustc"*.
///
/// `PATH` first is right **because every candidate is verified**: a shim fails
/// [`is_real_rustc`] and the search falls through to the baked path on its own.
/// `redox-bench`'s problem was a run-time probe with no baked candidate at all,
/// not the ordering. So the baked path is the fallback, not the answer, and it
/// is still only a candidate.
///
/// # `env!`, not `std::env::var`
///
/// `cargo::rustc-env` bakes the value in at **compile** time; it is not in the
/// process environment at run time. `redox-bench` read it with
/// `std::env::var` once, which therefore always missed — leaving `PATH`'s stub
/// as the only candidate under `blaze test`, and passing under cargo where
/// `PATH` happened to hold a real rustc and hid the mistake.
pub fn rustc_path() -> Option<PathBuf> {
  let from_path = std::env::var_os("PATH").into_iter().flat_map(|paths| {
    std::env::split_paths(&paths)
      .map(|dir| dir.join("rustc"))
      .collect::<Vec<_>>()
  });
  let baked = std::iter::once(PathBuf::from(env!("REDOX_RUSTC")));
  // `RUSTC` last. It is blaze's own shim under `blaze build`, so it is the
  // least trustworthy of the three and is here only so that a toolchain
  // reachable by no other means is still reachable.
  let from_env = std::env::var_os("RUSTC").map(PathBuf::from).into_iter();

  from_path
    .chain(baked)
    .chain(from_env)
    .find(|candidate| is_real_rustc(candidate))
}

#[cfg(test)]
mod tests {
  use super::rustc_path;
  use crate::rustc_path::is_real_rustc::is_real_rustc;

  /// The environment this suite runs in has a real rustc, since the suite was
  /// compiled by one.
  #[test]
  fn finds_a_rustc() {
    let found = rustc_path().expect("the toolchain should provide a rustc");
    assert!(is_real_rustc(&found), "{} is not rustc", found.display());
  }
}
