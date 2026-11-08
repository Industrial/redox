use std::path::{Path, PathBuf};

/// A temporary directory that removes itself.
///
/// Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) leaf 4. `redox check` needs a
/// file on disk to hand rustc, and the user did not ask for one — so it goes
/// somewhere temporary and is cleaned up, rather than littering the source
/// tree with a `.xrs` nobody asked for.
///
/// Keyed by process id *and* a counter, which is the lesson plan 04 learned
/// the hard way: keying on a name alone let two concurrent tests share a
/// directory, and one test's cleanup deleted a file the other was still using.
pub(crate) struct ScratchDir(PathBuf);

impl ScratchDir {
  pub(crate) fn new(tag: &str) -> Result<Self, String> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("redox-{tag}-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir)
      .map_err(|e| format!("error: cannot create {}: {e}", dir.display()))?;
    Ok(Self(dir))
  }

  pub(crate) fn path(&self) -> &Path {
    &self.0
  }
}

impl Drop for ScratchDir {
  fn drop(&mut self) {
    // A failure here is not worth reporting: the directory is temporary, and
    // the user came for a diagnostic, not for housekeeping news.
    std::fs::remove_dir_all(&self.0).ok();
  }
}

#[cfg(test)]
mod tests {
  use super::ScratchDir;

  #[test]
  fn creates_and_removes_itself() {
    let path = {
      let dir = ScratchDir::new("test").expect("should create");
      let path = dir.path().to_path_buf();
      assert!(path.is_dir());
      path
    };
    assert!(
      !path.exists(),
      "the directory should be gone after the drop"
    );
  }

  /// Two live scratch directories must not be the same directory.
  #[test]
  fn two_directories_do_not_collide() {
    let a = ScratchDir::new("test").expect("should create");
    let b = ScratchDir::new("test").expect("should create");
    assert_ne!(a.path(), b.path());
  }
}
