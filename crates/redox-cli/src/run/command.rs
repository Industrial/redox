pub(crate) mod state;

use crate::run::command::state::State;
/// One entry in Redox's command-line surface.
///
/// Plan 11 (`docs/16 - 11 - CLI Surface.md`) leaf 1. Doc 01 §27 fixes that
/// surface at six entries — `build`, `check`, `fmt`, `watch`, `--version`,
/// `--help` — because *"Cargo owns build, test, docs and dependencies"*, and
/// doc 01 §1.3 forbids replacing the build system.
///
/// Making it a table rather than a `match` is not tidiness. A CLI grows by
/// accident: every single addition looks reasonable, and the sum is the tool
/// doc 01 §1.3 says Redox must not become. Emerald's reached seventeen
/// subcommands (doc 02 §3), eleven of which Redox declines. One table that
/// dispatch, the usage text and the test all read is what makes a seventh
/// entry a deliberate act.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Command {
  /// As typed — `build`, or `--version`.
  pub(crate) name: &'static str,
  /// One line, for the usage text.
  pub(crate) summary: &'static str,
  /// What it does today.
  pub(crate) state: State,
}

impl Command {
  /// The argument forms that select this command.
  ///
  /// `--help` is also `-h` and `help`, and `--version` is also `-V`, because
  /// those are the conventions every other tool follows and a surface this
  /// small gains nothing by being surprising.
  pub(crate) fn matches(&self, arg: &str) -> bool {
    match self.name {
      "--help" => matches!(arg, "--help" | "-h" | "help"),
      "--version" => matches!(arg, "--version" | "-V"),
      name => arg == name,
    }
  }

  /// The message for a command that is declared but not yet implemented.
  pub(crate) fn not_yet(&self) -> Option<String> {
    match self.state {
      State::Ready => None,
      State::Planned { plan, id } => Some(format!(
        "error: `{}` is not implemented yet — it is plan {plan:02} (`{id}`)",
        self.name
      )),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::Command;
  use crate::run::command::state::State;

  const FMT: Command = Command {
    name: "fmt",
    summary: "format a .sdx in place",
    state: State::Planned {
      plan: 34,
      id: "formatter",
    },
  };

  const BUILD: Command = Command {
    name: "build",
    summary: "transpile",
    state: State::Ready,
  };

  #[test]
  fn a_ready_command_has_nothing_to_announce() {
    assert_eq!(BUILD.not_yet(), None);
  }

  #[test]
  fn a_planned_command_names_its_plan() {
    let message = FMT.not_yet().expect("fmt is planned");
    assert!(message.contains("not implemented yet"), "{message}");
    assert!(message.contains("plan 34"), "{message}");
    assert!(message.contains("formatter"), "{message}");
  }

  #[test]
  fn the_help_and_version_aliases_are_accepted() {
    let help = Command {
      name: "--help",
      summary: "",
      state: State::Ready,
    };
    for arg in ["--help", "-h", "help"] {
      assert!(help.matches(arg), "{arg}");
    }
    assert!(!help.matches("--helpful"));

    let version = Command {
      name: "--version",
      summary: "",
      state: State::Ready,
    };
    assert!(version.matches("--version") && version.matches("-V"));
    assert!(!version.matches("-v"), "lowercase -v is not a version flag");
  }

  #[test]
  fn an_ordinary_command_matches_only_itself() {
    assert!(BUILD.matches("build"));
    assert!(!BUILD.matches("builds") && !BUILD.matches("--build"));
  }
}
