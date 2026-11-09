use std::path::{Path, PathBuf};

/// What `redox watch` was pointed at, and where to watch.
///
/// Plan 12 (`docs/17 - 12 - Watcher.md`) leaf 1.
///
/// The `dir` is the directory handed to `notify`, and for a single file it is
/// that file's **parent**, not the file. This matters and is easy to get
/// wrong: editors overwhelmingly replace a file rather than writing it in
/// place — write a temporary, `rename` over the original — so a watch
/// registered on the file's inode stops receiving events after the first save.
/// The symptom is "the watcher worked once and then went quiet", which looks
/// like a race and is not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Target {
  /// The directory to register with the watcher.
  pub(crate) dir: PathBuf,
  /// The single file to compile, or `None` to compile the whole directory.
  pub(crate) file: Option<PathBuf>,
}

impl Target {
  /// Resolve `arg` to a watch target.
  pub(crate) fn resolve(arg: &Path) -> Result<Self, String> {
    if arg.is_dir() {
      return Ok(Self {
        dir: arg.to_path_buf(),
        file: None,
      });
    }

    if !arg.exists() {
      return Err(format!(
        "error: {} does not exist\n\n\
         `redox watch` takes a .sdx file or a directory of them.",
        arg.display()
      ));
    }

    let parent = arg.parent().filter(|p| !p.as_os_str().is_empty());
    Ok(Self {
      // A bare `main.sdx` has a parent of `""`, which is not a directory any
      // watcher will accept; `.` is the same place and is.
      dir: parent.unwrap_or(Path::new(".")).to_path_buf(),
      file: Some(arg.to_path_buf()),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::Target;
  use std::path::{Path, PathBuf};

  struct Dir(PathBuf);

  impl Dir {
    fn new(tag: &str) -> Self {
      use std::sync::atomic::{AtomicUsize, Ordering};
      static NEXT: AtomicUsize = AtomicUsize::new(0);
      let n = NEXT.fetch_add(1, Ordering::Relaxed);
      let dir = std::env::temp_dir().join(format!("redox-target-{tag}-{}-{n}", std::process::id()));
      std::fs::create_dir_all(&dir).expect("should create");
      Self(dir)
    }
  }

  impl Drop for Dir {
    fn drop(&mut self) {
      std::fs::remove_dir_all(&self.0).ok();
    }
  }

  #[test]
  fn a_directory_is_watched_whole() {
    let dir = Dir::new("whole");
    let target = Target::resolve(&dir.0).expect("should resolve");
    assert_eq!(target.dir, dir.0);
    assert_eq!(target.file, None, "a directory compiles all of itself");
  }

  /// The property that keeps the watcher alive across a save: the *parent* is
  /// registered, not the file.
  #[test]
  fn a_file_is_watched_through_its_parent() {
    let dir = Dir::new("parent");
    let file = dir.0.join("main.sdx");
    std::fs::write(&file, "main :: ()\nmain =\n  print 1\n").expect("should write");

    let target = Target::resolve(&file).expect("should resolve");
    assert_eq!(target.dir, dir.0, "the parent directory is what is watched");
    assert_eq!(target.file, Some(file));
  }

  #[test]
  fn a_missing_path_is_an_error_that_says_what_is_accepted() {
    let err = Target::resolve(Path::new("/nonexistent/main.sdx")).expect_err("should fail");
    assert!(err.contains("does not exist"), "{err}");
    assert!(err.contains(".sdx file or a directory"), "{err}");
  }

  /// A bare filename's parent is `""`, which no watcher accepts.
  #[test]
  fn a_bare_filename_resolves_to_the_current_directory() {
    let dir = Dir::new("bare");
    let file = dir.0.join("main.sdx");
    std::fs::write(&file, "main :: ()\nmain =\n  print 1\n").expect("should write");

    let previous = std::env::current_dir().expect("a cwd");
    std::env::set_current_dir(&dir.0).expect("should chdir");
    let target = Target::resolve(Path::new("main.sdx")).expect("should resolve");
    std::env::set_current_dir(previous).expect("should restore");

    assert_eq!(target.dir, Path::new("."), "not the empty path");
  }
}
