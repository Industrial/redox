pub(crate) mod command;
pub(crate) mod commands;
pub(crate) mod usage;

use crate::build_command::build_command;
use crate::check_command::check_command;
use crate::run::commands::lookup::lookup;
use crate::run::usage::usage;

/// Dispatch one invocation. Returns the message to print on failure.
///
/// Plan 11 (`docs/16 - 11 - CLI Surface.md`) leaf 3. The surface comes from
/// `COMMANDS`, which the usage text and `cli_surface.rs` also read, so the
/// three cannot disagree about what Redox accepts (§2.1).
///
/// Three outcomes for an argument, and keeping them distinct is the point of
/// §1.2: a command that runs, a command doc 01 §27 declares but a later plan
/// implements, and a word Redox does not know.
pub(crate) fn run(args: &[String]) -> Result<(), String> {
  let Some(first) = args.first().map(String::as_str) else {
    print!("{}", usage());
    return Ok(());
  };

  let Some(command) = lookup(first) else {
    return Err(format!("error: unknown command `{first}`\n\n{}", usage()));
  };

  // A declared command whose plan has not landed says so, and names the plan.
  if let Some(message) = command.not_yet() {
    return Err(message);
  }

  match command.name {
    "--help" => {
      print!("{}", usage());
      Ok(())
    }
    "--version" => {
      println!("redox {}", env!("CARGO_PKG_VERSION"));
      Ok(())
    }
    "build" => build_command(&args[1..]),
    "check" => check_command(&args[1..]),
    // Unreachable while `COMMANDS` and this match agree, and
    // `cli_surface.rs::every_ready_command_dispatches` is what keeps them
    // agreeing. A panic rather than a silent success: a `Ready` command with
    // no arm is a bug in Redox, not in the invocation.
    other => unreachable!("`{other}` is Ready in COMMANDS but has no dispatch arm"),
  }
}
