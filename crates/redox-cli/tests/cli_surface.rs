//! Redox's command-line surface is exactly what doc 01 §27 says it is.
//!
//! Plan 11 (`docs/16 - 11 - CLI Surface.md`) leaf 4.
//!
//! > `build`, `check`, `fmt`, `watch`, `--version`, `--help` … Cargo owns
//! > build, test, docs and dependencies.
//!
//! **The list below is a deliberate second copy.** A test that read `COMMANDS`
//! would pass for any table, which is no test at all; this one fails when the
//! table changes, so a seventh subcommand costs three edits — doc 01 §27, the
//! table, and a line here that says why it is here. Doc 02 §3 records
//! Emerald's CLI reaching seventeen subcommands, and nothing in its code said
//! when to stop. This is that something.

use std::process::Command;

/// Doc 01 §27's surface, written out.
const SURFACE: &[&str] = &["build", "check", "fmt", "watch", "--version", "--help"];

fn redox(args: &[&str]) -> std::process::Output {
  Command::new(env!("CARGO_BIN_EXE_redox"))
    .args(args)
    .output()
    .expect("should run the redox binary")
}

fn help_text() -> String {
  let out = redox(&["--help"]);
  String::from_utf8_lossy(&out.stdout).to_string()
}

/// Criteria 1 and 2: the surface is doc 01 §27's, and `--help` shows all of it.
#[test]
fn the_surface_is_exactly_doc_01_section_27() {
  let help = help_text();
  for name in SURFACE {
    assert!(
      help.contains(name),
      "`{name}` is in doc 01 §27 but not in --help:\n{help}"
    );
  }

  // And nothing else. Every `redox <word>` line in the usage must name one of
  // the six — a seventh would show up here.
  for line in help.lines().filter(|l| l.starts_with("  redox ")) {
    let named = line.trim_start().trim_start_matches("redox ");
    let name = named.split_whitespace().next().unwrap_or_default();
    assert!(
      SURFACE.contains(&name),
      "--help advertises `{name}`, which doc 01 §27 does not list. \
       If this is deliberate, doc 01 §27 and SURFACE both need editing:\n{line}"
    );
  }
}

/// Criterion 3: a declared-but-unimplemented command says so and names its
/// plan — the distinction plan 11 §1.2 exists to make.
#[test]
fn a_planned_command_names_its_plan() {
  // `fmt` is the only planned command left. `watch` was here too until plan 12
  // implemented it, and this test failing was how plan 12 learned it had
  // changed the surface — the table, the usage text and this test are kept in
  // step by each other rather than by memory.
  let (name, plan) = ("fmt", "plan 34");

  let out = redox(&[name]);
  let stderr = String::from_utf8_lossy(&out.stderr);
  assert!(!out.status.success(), "`{name}` should not succeed yet");
  assert!(
    stderr.contains("not implemented yet"),
    "`{name}` should say it is not implemented:\n{stderr}"
  );
  assert!(
    stderr.contains(plan),
    "`{name}` should name {plan}:\n{stderr}"
  );
  assert!(
    !stderr.contains("unknown command"),
    "`{name}` is declared by doc 01 §27, so it is not unknown:\n{stderr}"
  );
}

/// Criterion 4: an unknown command is reported as unknown, with the usage.
#[test]
fn an_unknown_command_is_reported_as_unknown() {
  let out = redox(&["frobnicate"]);
  let stderr = String::from_utf8_lossy(&out.stderr);
  assert!(!out.status.success());
  assert!(stderr.contains("unknown command `frobnicate`"), "{stderr}");
  assert!(
    stderr.contains("usage:"),
    "the usage should follow:\n{stderr}"
  );
  assert!(
    !stderr.contains("not implemented yet"),
    "a typo is not a planned feature:\n{stderr}"
  );
}

/// Criterion 5: `--help` and `--version` succeed, on stdout.
#[test]
fn help_and_version_succeed_on_stdout() {
  for args in [vec!["--help"], vec!["-h"], vec!["help"], vec![]] {
    let out = redox(&args);
    assert!(out.status.success(), "{args:?} should succeed");
    assert!(!out.stdout.is_empty(), "{args:?} should print to stdout");
    assert!(
      String::from_utf8_lossy(&out.stdout).contains("usage:"),
      "{args:?} should print the usage"
    );
  }

  for args in [["--version"], ["-V"]] {
    let out = redox(&args);
    assert!(out.status.success(), "{args:?} should succeed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.starts_with("redox "), "{stdout}");
    assert!(
      stdout
        .trim()
        .split(' ')
        .nth(1)
        .is_some_and(|v| v.contains('.')),
      "a version should look like one: {stdout}"
    );
  }
}

/// Every command the table calls `Ready` must actually dispatch.
///
/// `run`'s match panics on a `Ready` command with no arm, so this is what
/// turns that `unreachable!` into a test failure rather than a user-visible
/// crash. Invoked with no arguments, so a ready command fails on its *usage*
/// rather than succeeding — what matters is that it did not panic and did not
/// claim to be unimplemented.
#[test]
fn every_ready_command_dispatches() {
  for name in ["build", "check"] {
    let out = redox(&[name]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
      !stderr.contains("not implemented yet"),
      "`{name}` is Ready but reported as planned:\n{stderr}"
    );
    assert!(
      !stderr.contains("has no dispatch arm"),
      "`{name}` is Ready but has no dispatch arm:\n{stderr}"
    );
    assert!(
      stderr.contains("no input file"),
      "`{name}` with no arguments should ask for an input:\n{stderr}"
    );
  }
}

/// The surface names Cargo as the owner of everything it declines, so a reader
/// learns where `test`, `doc` and `add` went.
#[test]
fn the_usage_says_what_cargo_owns() {
  let help = help_text();
  assert!(help.contains("Cargo owns"), "{help}");
  for declined in ["test", "doc", "add", "publish", "init", "new"] {
    for line in help.lines().filter(|l| l.starts_with("  redox ")) {
      let name = line
        .trim_start()
        .trim_start_matches("redox ")
        .split_whitespace()
        .next()
        .unwrap_or_default();
      assert_ne!(
        name, declined,
        "`{declined}` is Cargo's, and doc 01 §27 declines it"
      );
    }
  }
}
