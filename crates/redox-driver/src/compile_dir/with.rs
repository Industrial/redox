use crate::cache::fingerprint::Fingerprint;
use crate::cache::reporter::CacheReporter;
use crate::cache::store::Store;
use crate::compile_dir::compile_file::compile_file;
use crate::compile_dir::compiled::Compiled;
use crate::compile_dir::sdx_in::sdx_in;
use redox_diag::Diagnostic;
use std::path::Path;

/// [`crate::compile_dir`], reporting each file's cache outcome.
///
/// Plan 10 (`docs/15 - 10 - Incremental Cache.md`) leaf 5. Lives at
/// `compile_dir/with.rs`, where doc 06 §2.3's parent-plus-stem rule makes
/// `compile_dir_with` the correct name rather than merely a permitted one.
///
/// The reporter exists
/// so a build script can print `[cache] greeting.sdx HIT` to **stderr** and a
/// test can collect the same events into a `Vec`. Doc 02 §7.1's phrasing is
/// that the cache has *"no opinion on stdout"*, and for a build script that is
/// not a style preference: stdout is Cargo's directive channel, so a cache
/// that printed there would corrupt the build it was speeding up.
///
/// The cache is consulted per file and keyed on the source bytes plus the
/// compiler's own fingerprint (plan 10 §1.1). A miss calls `compile_file`
/// verbatim — the rule the whole design rests on — so this function is correct
/// whether or not anything is cached.
pub fn compile_dir_with(
  src: &Path,
  out: &Path,
  reporter: &dyn CacheReporter,
) -> Result<Compiled, Vec<Diagnostic>> {
  let store = Store::new(out);
  let fingerprint = Fingerprint::current();

  let mut compiled = Compiled::default();
  let mut diagnostics = Vec::new();

  for source in sdx_in(src) {
    let label = source
      .file_name()
      .map_or_else(String::new, |n| n.to_string_lossy().to_string());

    match compile_file(&source, out, &store, &fingerprint) {
      Ok(outcome) => {
        reporter.report(&label, outcome.was_cached);
        compiled.inputs.push(source);
        compiled.outputs.push(outcome.written);
      }
      Err(mut errors) => {
        // A file that failed to compile was not served from cache, and saying
        // so keeps the reported events in step with the files attempted.
        reporter.report(&label, false);
        diagnostics.append(&mut errors);
      }
    }
  }

  if diagnostics.is_empty() {
    Ok(compiled)
  } else {
    Err(diagnostics)
  }
}
