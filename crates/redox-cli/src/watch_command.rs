pub(crate) mod is_change;
pub(crate) mod is_relevant;
pub(crate) mod report;
pub(crate) mod run_build;
pub(crate) mod sources;
pub(crate) mod target;

use crate::watch_command::is_change::is_change;
use crate::watch_command::is_relevant::is_relevant;
use crate::watch_command::report::report;
use crate::watch_command::target::Target;
use notify::{RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;

/// `redox watch <file.sdx | dir>`.
///
/// Plan 12 (`docs/17 - 12 - Watcher.md`) leaf 3. Builds once, then rebuilds on
/// every relevant change, and **keeps going when a build fails**.
///
/// There is no debounce window and no mtime comparison: plan 10's cache makes
/// an unchanged file a content hash rather than a parse and an emit, so the
/// watcher needs no timing heuristic to avoid redundant work. `watch.rs`
/// asserts that absence.
///
/// What the cache does **not** buy is freedom from caring which events matter
/// — plan 12 §1.1 claimed it did, and §10.1 records the correction. A cache
/// makes a redundant compile cheap; it cannot break a loop whose trigger is
/// the compile itself. Hence [`is_change`], which rejects the build's own read
/// of its own input.
///
/// Doc 01 §28 bounds this: it watches and rebuilds. It does not run tests,
/// resolve dependencies, or sequence tasks — that is a build system, and doc
/// 01 §1.3 forbids replacing Cargo's. `cargo watch -x test` already exists and
/// composes with this.
pub(crate) fn watch_command(args: &[String]) -> Result<(), String> {
  let input = args.iter().find(|a| !a.starts_with('-')).ok_or_else(|| {
    "error: no input file\n\n`redox watch` takes a .sdx file or a directory.".to_string()
  })?;
  let target = Target::resolve(Path::new(input))?;
  let plain = args.iter().any(|a| a == "--plain");

  // Build before watching, so an already-broken file is reported now rather
  // than after the user's next keystroke.
  report(&target, plain);

  let (tx, rx) = mpsc::channel();
  let mut watcher = notify::recommended_watcher(move |event| {
    // A send failure means the receiver is gone, which means the process is
    // ending; there is nothing useful to do about it here.
    let _ = tx.send(event);
  })
  .map_err(|e| format!("error: cannot start the file watcher: {e}"))?;

  watcher
    .watch(&target.dir, RecursiveMode::Recursive)
    .map_err(|e| format!("error: cannot watch {}: {e}", target.dir.display()))?;

  eprintln!("watching {} — press ctrl-c to stop", target.dir.display());

  for received in rx {
    match received {
      Ok(event) => {
        // Two filters, and both are necessary. `is_change` rejects the
        // build's own *read* of the source, which is a `.sdx` path and would
        // otherwise retrigger the build forever (plan 12 §10.1 — measured at
        // 97,442 builds in six seconds). `is_relevant` rejects the generated
        // `.xrs` it writes, which is a real change to an irrelevant path.
        if is_change(&event.kind) && event.paths.iter().any(|p| is_relevant(p)) {
          report(&target, plain);
        }
      }
      // A watcher error — a deleted directory, an exhausted inotify watch
      // limit — is reported and survived, for the same reason a compile error
      // is: exiting means the user fixes the cause and nothing happens.
      Err(e) => eprintln!("error: the file watcher reported: {e}"),
    }
  }

  Ok(())
}
