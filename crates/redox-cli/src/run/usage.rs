use crate::run::command::state::State;
use crate::run::commands::COMMANDS;
use std::fmt::Write as _;

/// The one-screen usage message, **generated from the command table**.
///
/// Plan 11 (`docs/16 - 11 - CLI Surface.md`) leaf 2. This was a hand-written
/// `const` until plan 11, which made it a second independent list of the same
/// six commands the dispatch `match` listed. Plan 07 had to edit both and
/// nothing would have objected if it had edited only one — leaving either a
/// subcommand that works and is undocumented, or one that is documented and
/// does not work. No test covered either case (plan 11 §2.1).
///
/// Generating it removes the possibility rather than that instance.
///
/// A planned command appears with its plan, so `--help` tells the truth about
/// what works rather than advertising six commands when four run.
pub(crate) fn usage() -> String {
  let mut out = String::from(
    "redox — a MoonScript-inspired surface for Rust\n\
     \n\
     usage:\n",
  );

  let width = COMMANDS.iter().map(|c| c.name.len()).max().unwrap_or(0);
  for command in COMMANDS {
    let name = command.name;
    let _ = write!(out, "  redox {name:<width$}  {}", command.summary);
    if let State::Planned { plan, id } = command.state {
      let _ = write!(out, " (plan {plan:02}, `{id}`)");
    }
    out.push('\n');
  }

  out.push_str(
    "\n\
     Cargo owns build, test, docs and dependencies (doc 01 §27).\n",
  );
  out
}

#[cfg(test)]
mod tests {
  use super::usage;
  use crate::run::commands::COMMANDS;

  /// Criterion 2: a declared command cannot be hidden from `--help`.
  #[test]
  fn every_command_appears() {
    let text = usage();
    for command in COMMANDS {
      assert!(
        text.contains(command.name),
        "{} is missing from the usage text:\n{text}",
        command.name
      );
      assert!(
        text.contains(command.summary),
        "{} summary missing",
        command.name
      );
    }
  }

  /// **No command advertises a plan**, because every declared command is now
  /// built.
  ///
  /// This asserted the opposite until plan 34 — it required `fmt` to say
  /// *"plan 34"* — and its own comment recorded the pattern: *"`watch` was
  /// listed here until plan 12 implemented it, and this test failed when it
  /// did, which is the usage text and the table staying in step."* `fmt` was
  /// the last planned command, so the assertion inverts rather than moving to
  /// another name.
  ///
  /// Plan 11 §2.1's guarantee is unchanged and still enforced: a `Ready`
  /// command must not advertise a plan, and now none may, because none is
  /// planned. A future row that declares a command ahead of itself makes this
  /// fail and has to say so.
  #[test]
  fn no_command_advertises_a_plan() {
    let text = usage();
    assert!(
      !text.contains("plan "),
      "every command is implemented, so none should name a plan:\n{text}"
    );
  }

  /// The usage text is one screen. A tool whose help needs scrolling has a
  /// surface problem, so the length is an assertion rather than a hope.
  #[test]
  fn fits_on_one_screen() {
    let lines = usage().lines().count();
    assert!(
      lines <= 24,
      "usage is {lines} lines, which no longer fits a terminal"
    );
  }

  #[test]
  fn names_cargo_as_the_owner_of_everything_else() {
    assert!(usage().contains("Cargo owns"));
  }
}
