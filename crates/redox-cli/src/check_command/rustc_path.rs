pub(crate) mod is_real_rustc;

use crate::check_command::rustc_path::is_real_rustc::is_real_rustc;
use std::path::PathBuf;

/// A rustc that behaves like rustc, resolved at run time.
///
/// Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) leaf 5. Lives under
/// `check_command` because that is its only caller, which is what doc 06 §2.1
/// requires of a construct with no children of its own; the day `fmt` or
/// `watch` also needs a verified toolchain path is the day it earns a home
/// further up.
///
/// `PATH` first, then the `REDOX_RUSTC` that `build.rs` baked in — in that
/// order, because a shipped binary must not depend on the rustc that happened
/// to be installed when it was compiled. The baked path is the fallback, not
/// the answer.
///
/// Every candidate is verified by [`is_real_rustc`] rather than trusted by its
/// name; that function records why.
pub(crate) fn rustc_path() -> Option<PathBuf> {
  let from_path = std::env::var_os("PATH").into_iter().flat_map(|paths| {
    std::env::split_paths(&paths)
      .map(|dir| dir.join("rustc"))
      .collect::<Vec<_>>()
  });
  let baked = std::iter::once(PathBuf::from(env!("REDOX_RUSTC")));

  from_path
    .chain(baked)
    .find(|candidate| is_real_rustc(candidate))
}

#[cfg(test)]
mod tests {
  use super::rustc_path;
  use crate::check_command::rustc_path::is_real_rustc::is_real_rustc;

  /// The environment this suite runs in has a real rustc, since the suite was
  /// compiled by one.
  #[test]
  fn finds_a_rustc() {
    let found = rustc_path().expect("the toolchain should provide a rustc");
    assert!(is_real_rustc(&found), "{} is not rustc", found.display());
  }
}
