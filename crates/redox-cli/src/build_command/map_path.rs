use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Where the source map sits for a given output file.
///
/// Plan 06 (`docs/11 - 06 - Source Map.md`) leaf 4. The whole file name plus
/// `.map`, so `main.xrs` has `main.xrs.map` — *not* `Path::with_extension`,
/// which would replace `.xrs` and give `main.map`. Keeping the `.xrs` visible
/// means a directory listing says which generated file a map belongs to, and
/// a map can never collide with the output of a differently-suffixed input.
pub(crate) fn map_path(output: &Path) -> PathBuf {
  let mut name = output
    .file_name()
    .map_or_else(OsString::new, OsString::from);
  name.push(".map");
  output.with_file_name(name)
}

#[cfg(test)]
mod tests {
  use super::map_path;
  use std::path::Path;

  #[test]
  fn appends_rather_than_replacing() {
    assert_eq!(map_path(Path::new("main.xrs")), Path::new("main.xrs.map"));
  }

  #[test]
  fn keeps_the_directory() {
    assert_eq!(
      map_path(Path::new("/tmp/out/main.xrs")),
      Path::new("/tmp/out/main.xrs.map")
    );
  }

  #[test]
  fn handles_a_name_with_no_extension() {
    assert_eq!(map_path(Path::new("out/main")), Path::new("out/main.map"));
  }
}
