use crate::run::command::Command;
use crate::run::commands::COMMANDS;

/// The command `arg` selects, if any.
///
/// Plan 11 (`docs/16 - 11 - CLI Surface.md`) leaf 3. The single way an
/// argument becomes a command, so dispatch cannot recognise something the
/// usage text does not — the defect §2.1 describes, where `USAGE` and the
/// dispatch `match` were two independent lists that nothing kept in step.
pub(crate) fn lookup(arg: &str) -> Option<&'static Command> {
  COMMANDS.iter().find(|c| c.matches(arg))
}

#[cfg(test)]
mod tests {
  use super::lookup;
  use crate::run::commands::COMMANDS;

  #[test]
  fn every_declared_command_is_reachable_by_its_own_name() {
    for command in COMMANDS {
      let found = lookup(command.name).unwrap_or_else(|| panic!("{} not found", command.name));
      assert_eq!(found.name, command.name);
    }
  }

  #[test]
  fn an_unknown_argument_selects_nothing() {
    assert!(lookup("frobnicate").is_none());
    assert!(lookup("").is_none());
  }

  /// Two entries matching the same argument would make dispatch depend on
  /// table order, which is the kind of thing nobody notices until the order
  /// changes.
  #[test]
  fn no_two_commands_claim_the_same_argument() {
    for arg in [
      "build",
      "check",
      "fmt",
      "watch",
      "--version",
      "-V",
      "--help",
      "-h",
      "help",
    ] {
      let claimants = COMMANDS.iter().filter(|c| c.matches(arg)).count();
      assert_eq!(claimants, 1, "{arg} is claimed {claimants} times");
    }
  }
}
