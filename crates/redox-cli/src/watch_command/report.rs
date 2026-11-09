use crate::watch_command::run_build::run_build;
use crate::watch_command::sources::sources;
use crate::watch_command::target::Target;

/// Compile the target once, printing whatever happened.
///
/// Plan 12 (`docs/17 - 12 - Watcher.md`) leaf 3. Called before the loop starts
/// and on every relevant event, so an already-broken file is reported
/// immediately rather than after the user's next keystroke.
pub(crate) fn report(target: &Target, plain: bool) {
  match &target.file {
    Some(file) => run_build(&file.display().to_string(), plain),
    // A directory target compiles every `.sdx` in it, one at a time, because
    // `build_command` takes one file. Plan 10's cache makes the files that did
    // not change cost a hash each.
    None => {
      for source in sources(&target.dir) {
        run_build(&source, plain);
      }
    }
  }
}
