pub(crate) mod usage;

use crate::build_command::build_command;
use crate::run::usage::USAGE;

/// Dispatch one invocation. Returns the message to print on failure.
pub(crate) fn run(args: &[String]) -> Result<(), String> {
  match args.first().map(String::as_str) {
    None | Some("--help" | "-h" | "help") => {
      print!("{USAGE}");
      Ok(())
    }
    Some("--version" | "-V") => {
      println!("redox {}", env!("CARGO_PKG_VERSION"));
      Ok(())
    }
    Some("build") => build_command(&args[1..]),
    Some(other) => Err(format!("error: unknown command `{other}`\n\n{USAGE}")),
  }
}
