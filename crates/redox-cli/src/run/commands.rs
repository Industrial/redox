pub(crate) mod lookup;

use crate::run::command::Command;
use crate::run::command::state::State;

/// Redox's entire command-line surface.
///
/// Plan 11 (`docs/16 - 11 - CLI Surface.md`) leaf 1. **Six entries, because
/// doc 01 §27 says six**:
///
/// > `build`, `check`, `fmt`, `watch`, `--version`, `--help` … Cargo owns
/// > build, test, docs and dependencies.
///
/// A seventh needs a change to doc 01 §27, to this table, *and* to
/// `cli_surface.rs`'s literal list — which is three deliberate acts, and the
/// point. Doc 02 §3 records Emerald's CLI reaching seventeen subcommands;
/// nothing in its code said when to stop.
///
/// Plan 08 already turned one addition away: doc 04 proposed a `redox test`
/// entry point and plan 08 §6.1 declined it, because the corpus is a cargo
/// integration test and `cargo test` runs it. That happened because the rule
/// was written down, and this table is the rule made executable.
pub(crate) const COMMANDS: &[Command] = &[
  Command {
    name: "build",
    summary: "transpile a .sdx to .xrs",
    state: State::Ready,
  },
  Command {
    name: "check",
    summary: "transpile, then report rustc's errors against the .sdx",
    state: State::Ready,
  },
  Command {
    name: "fmt",
    // Plural, because plan 34 §1.5's gate is `redox fmt --check examples/*.sdx`
    // and a one-file command would make that a shell loop nobody checks.
    summary: "format .sdx files in place, or --check them",
    state: State::Ready,
  },
  Command {
    name: "watch",
    summary: "rebuild on change",
    state: State::Ready,
  },
  Command {
    name: "--version",
    summary: "print the version",
    state: State::Ready,
  },
  Command {
    name: "--help",
    summary: "print this message",
    state: State::Ready,
  },
];

#[cfg(test)]
mod tests {
  use super::COMMANDS;

  #[test]
  fn every_command_has_a_summary() {
    for command in COMMANDS {
      assert!(
        !command.summary.is_empty(),
        "{} has no summary",
        command.name
      );
    }
  }

  /// Doc 01 §27 fixes the surface at six entries. This is the table's own
  /// count; `cli_surface.rs` holds the list, deliberately duplicated.
  #[test]
  fn the_surface_has_six_entries() {
    assert_eq!(
      COMMANDS.len(),
      6,
      "doc 01 §27 lists six commands; adding a seventh is a decision, not a patch"
    );
  }
}
