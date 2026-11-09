use std::path::Path;

/// Whether a changed path warrants a compile.
///
/// Plan 12 (`docs/17 - 12 - Watcher.md`) leaf 2.
///
/// **Not a correctness filter.** Plan 10's cache makes a spurious compile cost
/// a hash rather than a parse, so letting an irrelevant event through is
/// merely wasteful (plan 12 §1.1). The filter exists for the *output*: some
/// editors touch a temporary file on every keystroke, and
/// `[cache] a.sdx HIT` printed thirty times a second is noise a user has to
/// learn to ignore.
///
/// Which is why this is a deny-list of known editor artefacts rather than an
/// allow-list of blessed events: being wrong about an unfamiliar editor's
/// temporary file costs a redundant line of output, while being wrong in the
/// other direction would silently stop rebuilding.
pub(crate) fn is_relevant(path: &Path) -> bool {
  let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_string()) else {
    return false;
  };

  // Vim writes `4913` to test whether a directory is writable, and leaves
  // `.swp`/`.swx` files; Emacs uses `#name#` and `.#name`; many editors and
  // tools use a `~` suffix or a `.tmp` extension.
  if name == "4913"
    || name.ends_with('~')
    || name.starts_with(".#")
    || (name.starts_with('#') && name.ends_with('#'))
  {
    return false;
  }

  let extension = path.extension().map(|e| e.to_string_lossy().to_string());
  match extension.as_deref() {
    Some("sdx") => !name.starts_with('.'),
    _ => false,
  }
}

#[cfg(test)]
mod tests {
  use super::is_relevant;
  use std::path::Path;

  #[test]
  fn a_sdx_file_is_relevant() {
    assert!(is_relevant(Path::new("src/main.sdx")));
    assert!(is_relevant(Path::new("main.sdx")));
  }

  #[test]
  fn another_extension_is_not() {
    for name in ["main.rs", "main.xrs", "Cargo.toml", "README.md", "main"] {
      assert!(!is_relevant(Path::new(name)), "{name}");
    }
  }

  /// `.xrs` is Redox's *output*. Compiling because the output changed would
  /// be a loop that feeds itself.
  #[test]
  fn the_generated_output_is_not_relevant() {
    assert!(!is_relevant(Path::new("main.xrs")));
    assert!(!is_relevant(Path::new("main.xrs.map")));
  }

  #[test]
  fn editor_temporaries_are_ignored() {
    for name in [
      "4913",          // vim's writability probe
      "main.sdx~",     // a backup suffix
      ".main.sdx.swp", // vim's swap file
      ".#main.sdx",    // emacs' lock file
      "#main.sdx#",    // emacs' auto-save
      ".main.sdx",     // a dotfile shadowing the source
    ] {
      assert!(!is_relevant(Path::new(name)), "{name} should be ignored");
    }
  }

  #[test]
  fn a_path_with_no_file_name_is_not_relevant() {
    assert!(!is_relevant(Path::new("/")));
    assert!(!is_relevant(Path::new("..")));
  }

  /// A deny-list errs toward compiling. Asserting that explicitly, because the
  /// opposite default — an allow-list — would silently stop rebuilding for an
  /// editor nobody tested.
  #[test]
  fn an_unfamiliar_name_still_compiles_if_it_is_a_sdx() {
    assert!(is_relevant(Path::new("weird-editor-artifact.sdx")));
  }
}
