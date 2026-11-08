use std::path::PathBuf;

/// What a [`super::compile_dir`] run produced.
///
/// Plan 09 (`docs/14 - 09 - Cargo Integration.md`) leaf 2.
///
/// `inputs` exists for Cargo. A build script must emit
/// `cargo::rerun-if-changed=<path>` for every file it read, or Cargo reruns it
/// only when the crate's own files change — so editing a `.sdx` would not
/// rebuild, and the user would be running stale code with no indication. That
/// is the silent-staleness failure doc 02 §6.6 warns of in option (b), and it
/// would reappear in option (a) without this.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Compiled {
  /// Every `.sdx` read, in the order compiled.
  pub inputs: Vec<PathBuf>,
  /// Every `.rs` written, positionally matching [`Compiled::inputs`].
  pub outputs: Vec<PathBuf>,
}

impl Compiled {
  /// The `cargo::rerun-if-changed` lines a build script should print.
  ///
  /// Returned as lines rather than printed here: a library that writes to
  /// stdout is a library that cannot be tested, and Cargo's directives are
  /// stdout-sensitive.
  pub fn rerun_directives(&self) -> Vec<String> {
    self
      .inputs
      .iter()
      .map(|p| format!("cargo::rerun-if-changed={}", p.display()))
      .collect()
  }

  pub fn is_empty(&self) -> bool {
    self.inputs.is_empty()
  }
}

#[cfg(test)]
mod tests {
  use super::Compiled;
  use std::path::PathBuf;

  #[test]
  fn names_every_input_for_cargo() {
    let compiled = Compiled {
      inputs: vec![PathBuf::from("src/a.sdx"), PathBuf::from("src/b.sdx")],
      outputs: vec![PathBuf::from("out/a.rs"), PathBuf::from("out/b.rs")],
    };
    assert_eq!(
      compiled.rerun_directives(),
      vec![
        "cargo::rerun-if-changed=src/a.sdx",
        "cargo::rerun-if-changed=src/b.sdx"
      ]
    );
  }

  #[test]
  fn an_empty_run_asks_for_no_reruns() {
    assert!(Compiled::default().rerun_directives().is_empty());
    assert!(Compiled::default().is_empty());
  }
}
