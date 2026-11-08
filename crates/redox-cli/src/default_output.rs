use std::path::{Path, PathBuf};

/// The output path for an input with no explicit `-o`.
///
/// `main.sdx` becomes `main.xrs` (doc 01 §3.4). A path with any other
/// extension, or none, *gains* `.xrs` rather than losing what it has.
pub(crate) fn default_output(input: &Path) -> PathBuf {
  if input.extension().is_some_and(|e| e == "sdx") {
    input.with_extension("xrs")
  } else {
    let mut name = input.as_os_str().to_os_string();
    name.push(".xrs");
    PathBuf::from(name)
  }
}

#[cfg(test)]
mod tests {
  use super::default_output;
  use std::path::{Path, PathBuf};

  #[test]
  fn maps_sdx_to_xrs() {
    assert_eq!(
      default_output(Path::new("main.sdx")),
      PathBuf::from("main.xrs")
    );
    assert_eq!(
      default_output(Path::new("a/b/main.sdx")),
      PathBuf::from("a/b/main.xrs")
    );
  }

  #[test]
  fn appends_to_an_unexpected_extension_rather_than_replacing_it() {
    assert_eq!(
      default_output(Path::new("notes.txt")),
      PathBuf::from("notes.txt.xrs")
    );
  }
}
