use crate::cache::store::cached::Cached;
use crate::diagnose::io_diagnostic::io_diagnostic;
use redox_diag::Diagnostic;
use std::path::{Path, PathBuf};

/// Write the generated Rust and its map, whatever their provenance.
///
/// Plan 10 (`docs/15 - 10 - Incremental Cache.md`) leaf 5. Shared by the cache
/// hit and miss paths **so the two cannot diverge**. If they wrote different
/// bytes, a cache hit would change the build — precisely what plan 10 §1
/// forbids — and the difference would appear only on a second run, which is
/// the hardest kind of bug to reproduce.
///
/// The hit path writes rather than assuming last run's files survive.
/// `OUT_DIR` does persist, but so does a half-finished build, and a write of
/// a few kilobytes makes the question moot.
pub(crate) fn write_outputs(
  source: &Path,
  out: &Path,
  path: &str,
  content: &Cached,
) -> Result<PathBuf, Vec<Diagnostic>> {
  let stem = source.file_stem().map_or_else(
    || "generated".to_string(),
    |s| s.to_string_lossy().to_string(),
  );
  let target = out.join(format!("{stem}.rs"));

  std::fs::create_dir_all(out).map_err(|e| {
    vec![io_diagnostic(
      path,
      format!("cannot create {}: {e}", out.display()),
    )]
  })?;
  std::fs::write(&target, &content.rust).map_err(|e| {
    vec![io_diagnostic(
      path,
      format!("cannot write {}: {e}", target.display()),
    )]
  })?;

  let mut map_name = target.file_name().expect("a name").to_os_string();
  map_name.push(".map");
  let map_path = target.with_file_name(map_name);
  std::fs::write(&map_path, &content.map).map_err(|e| {
    vec![io_diagnostic(
      path,
      format!("cannot write {}: {e}", map_path.display()),
    )]
  })?;

  Ok(target)
}
