use std::path::Path;
use std::process::Command;

/// Whether `candidate` answers `--version` the way rustc itself does.
///
/// Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) leaf 5. This check is not
/// defensive programming, it is a scar. Three times now something has answered
/// to the name `rustc` and not been it:
///
/// - plan 01 found NixOS's `stub-ld` shim on `PATH` inside `blaze test`;
/// - plan 03 found `BLAZE_SCRIPT_RUSTC` naming something that was not blaze's
///   rustc either;
/// - plan 03 found `RUSTC` itself pointing at blaze's wrapper, which proxies
///   for its own build graph and fails on an arbitrary file.
///
/// Each cost an hour to a confusing error from a program that was not the
/// compiler. The rule those earned: **never trust a name, check the banner.**
pub(crate) fn is_real_rustc(candidate: &Path) -> bool {
  if !candidate.is_file() {
    return false;
  }
  Command::new(candidate)
    .arg("--version")
    .output()
    .is_ok_and(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).starts_with("rustc "))
}

#[cfg(test)]
mod tests {
  use super::is_real_rustc;
  use std::path::Path;

  /// A shell exits zero for `--version` and prints something that is not a
  /// rustc banner, which is precisely the shape of the shims that caused this
  /// function to exist.
  #[test]
  fn rejects_a_program_that_is_not_rustc() {
    assert!(!is_real_rustc(Path::new("/bin/sh")));
  }

  #[test]
  fn rejects_a_path_that_does_not_exist() {
    assert!(!is_real_rustc(Path::new("/nonexistent/rustc")));
  }

  #[test]
  fn rejects_a_directory() {
    assert!(!is_real_rustc(Path::new("/tmp")));
  }
}
