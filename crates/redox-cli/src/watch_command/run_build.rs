use crate::build_command::build_command;

/// Build one file, printing whatever happened, and never failing.
///
/// Plan 12 (`docs/17 - 12 - Watcher.md`) §1.2 — the property that makes a
/// watcher worth having. A watcher that exits on a compile error is worse than
/// none: the user fixes the typo and nothing happens, because the process died
/// three seconds ago.
///
/// The build goes through `build_command`, the same path `redox build` takes,
/// so the loop's diagnostics cannot drift from the ones a direct build
/// produces. A watcher-specific error format would be a second renderer to
/// keep in step.
///
/// Output goes to **stderr**, including the success line: a watch loop's
/// progress is not the program's output, and keeping stdout clear means
/// `redox watch` composes with a pipe.
pub(crate) fn run_build(input: &str, plain: bool) {
  let mut args = vec![input.to_string()];
  if plain {
    args.push("--plain".to_string());
  }

  match build_command(&args) {
    Ok(()) => eprintln!("built {input}"),
    Err(message) => eprintln!("{message}"),
  }
}
