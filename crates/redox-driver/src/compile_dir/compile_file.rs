pub(crate) mod outcome;
pub(crate) mod write_outputs;

use crate::cache::cache_key::cache_key;
use crate::cache::fingerprint::Fingerprint;
use crate::cache::store::Store;
use crate::cache::store::cached::Cached;
use crate::compile_dir::compile_file::outcome::Outcome;
use crate::compile_dir::compile_file::write_outputs::write_outputs;
use crate::diagnose::diagnose;
use crate::diagnose::io_diagnostic::io_diagnostic;
use redox_diag::Diagnostic;
use std::path::Path;

/// Compile one `.sdx` into `<out>/<stem>.rs`, with its map beside it.
///
/// Plan 09 (`docs/14 - 09 - Cargo Integration.md`) leaf 1; plan 10 added the
/// cache. Returns what was written and whether the cache supplied it.
///
/// `.rs` rather than `.xrs` because that is the extension rustc accepts; the
/// `.xrs` stays the canonical human-facing artifact `redox build` writes for
/// reading (doc 02 §6.6). The map sidecar goes beside the generated file under
/// the same whole-name-plus-`.map` rule the CLI uses, so plan 07's remapping
/// finds it the same way through a cargo build.
///
/// **The cache changes only whether parse and emit run**, never what is
/// written. A hit writes the same bytes a miss would — that is plan 10 §1's
/// rule, and the reason the hit path still writes the files rather than
/// assuming last run's outputs survive: `OUT_DIR` persists, but so does a
/// half-finished build, and the cheap write makes the question moot.
pub(crate) fn compile_file(
  source: &Path,
  out: &Path,
  store: &Store,
  fingerprint: &Fingerprint,
) -> Result<Outcome, Vec<Diagnostic>> {
  let path = source.display().to_string();
  let text = std::fs::read_to_string(source)
    .map_err(|e| vec![io_diagnostic(&path, format!("cannot read {path}: {e}"))])?;

  let key = cache_key(&text, fingerprint);
  if let Some(hit) = store.get(&key) {
    let written = write_outputs(source, out, &path, &hit)?;
    return Ok(Outcome {
      written,
      was_cached: true,
    });
  }

  let program =
    redox_parser::parse_source(&text).map_err(|failure| diagnose(source, &text, &failure))?;
  let (rust, map) = redox_emit::emit_with_map(&program, &text, &path);
  let fresh = Cached {
    rust,
    map: redox_diag::render_map(&map),
  };

  let written = write_outputs(source, out, &path, &fresh)?;
  store.put(&key, &fresh);
  Ok(Outcome {
    written,
    was_cached: false,
  })
}
