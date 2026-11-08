use crate::diagnose::diagnose;
use crate::diagnose::io_diagnostic::io_diagnostic;
use redox_diag::Diagnostic;
use std::path::{Path, PathBuf};

/// Compile one `.sdx` into `<out>/<stem>.rs`, with its map beside it.
///
/// Plan 09 (`docs/14 - 09 - Cargo Integration.md`) leaf 1. Returns the path
/// written.
///
/// `.rs` rather than `.xrs` because that is the extension rustc accepts; the
/// `.xrs` stays the canonical human-facing artifact `redox build` writes for
/// reading (doc 02 §6.6). The map sidecar goes beside the generated file under
/// the same whole-name-plus-`.map` rule the CLI uses, so plan 07's remapping
/// finds it the same way through a cargo build.
pub(crate) fn compile_file(source: &Path, out: &Path) -> Result<PathBuf, Vec<Diagnostic>> {
  let path = source.display().to_string();
  let text = std::fs::read_to_string(source)
    .map_err(|e| vec![io_diagnostic(&path, format!("cannot read {path}: {e}"))])?;

  let program =
    redox_parser::parse_source(&text).map_err(|failure| diagnose(source, &text, &failure))?;
  let (rust, map) = redox_emit::emit_with_map(&program, &text, &path);

  let stem = source.file_stem().map_or_else(
    || "generated".to_string(),
    |s| s.to_string_lossy().to_string(),
  );
  let target = out.join(format!("{stem}.rs"));

  std::fs::create_dir_all(out).map_err(|e| {
    vec![io_diagnostic(
      &path,
      format!("cannot create {}: {e}", out.display()),
    )]
  })?;
  std::fs::write(&target, &rust).map_err(|e| {
    vec![io_diagnostic(
      &path,
      format!("cannot write {}: {e}", target.display()),
    )]
  })?;

  let mut map_name = target.file_name().expect("a name").to_os_string();
  map_name.push(".map");
  let map_path = target.with_file_name(map_name);
  std::fs::write(&map_path, redox_diag::render_map(&map)).map_err(|e| {
    vec![io_diagnostic(
      &path,
      format!("cannot write {}: {e}", map_path.display()),
    )]
  })?;

  Ok(target)
}
