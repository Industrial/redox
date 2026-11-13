//! `voltaic watch` rebuilds on change, and survives a failure.
//!
//! Plan 12 (`docs/17 - 12 - Watcher.md`) leaf 4.
//!
//! A watcher is timing-dependent, so the proof is deliberately split (plan 12
//! §3): `is_relevant` and `Target` are pure logic with their own unit tests,
//! and the two properties that genuinely need the operating system get one
//! end-to-end test each. The alternative — asserting the whole thing through
//! the filesystem — produces a test that fails on a loaded machine and teaches
//! people to re-run the suite.
//!
//! Both end-to-end tests spawn the real binary, watch with real `notify`, and
//! poll for an outcome with a generous ceiling rather than sleeping for a
//! fixed interval. A fixed sleep is the same timing heuristic plan 12 §1.1
//! removed from the watcher itself, and it would be no better in a test.

mod support;

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

/// How long to wait for the watcher to react before calling it a failure.
///
/// Generous on purpose: this bounds a *failure*, not a success. A correct
/// watcher usually reacts in milliseconds, and nothing waits the full ceiling
/// unless something is broken.
const CEILING: std::time::Duration = std::time::Duration::from_secs(20);

/// A spawned `voltaic watch`, killed on drop.
struct Watching {
  child: Child,
  dir: PathBuf,
}

impl Drop for Watching {
  fn drop(&mut self) {
    self.child.kill().ok();
    self.child.wait().ok();
    std::fs::remove_dir_all(&self.dir).ok();
  }
}

impl Watching {
  fn start(tag: &str, initial: &str) -> Self {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("voltaic-watch-{tag}-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("should create the scratch directory");
    std::fs::write(dir.join("main.sdx"), initial).expect("should write the source");

    let child = Command::new(env!("CARGO_BIN_EXE_voltaic"))
      .arg("watch")
      .arg(dir.join("main.sdx"))
      .arg("--plain")
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .expect("should spawn voltaic watch");

    Self { child, dir }
  }

  fn write(&self, text: &str) {
    std::fs::write(self.dir.join("main.sdx"), text).expect("should rewrite the source");
  }

  /// Rewrite the source the way an editor does: write a temporary, then
  /// `rename` over the original.
  ///
  /// This is criterion 5, and it is a different operation from `write`.
  /// `fs::write` truncates the existing file, so the inode survives; a rename
  /// replaces it. A watcher registered on the *file* stops receiving events
  /// after the first such save, which is why `Target` watches the parent
  /// directory — and this is what proves it.
  fn replace(&self, text: &str) {
    let temporary = self.dir.join("main.sdx.tmp-editor");
    std::fs::write(&temporary, text).expect("should write the temporary");
    std::fs::rename(&temporary, self.dir.join("main.sdx")).expect("should rename over the source");
  }

  /// Wait until `.xrs` contains `needle`, or the ceiling elapses.
  ///
  /// Polls the *output file* rather than parsing the child's stderr: stderr
  /// would mean reading a pipe that may block, and the artifact is the thing
  /// the user actually cares about.
  fn wait_for_output(&self, needle: &str) -> Result<String, String> {
    let xrs = self.dir.join("main.xrs");
    let deadline = std::time::Instant::now() + CEILING;
    let mut last = String::new();
    while std::time::Instant::now() < deadline {
      if let Ok(text) = std::fs::read_to_string(&xrs) {
        if text.contains(needle) {
          return Ok(text);
        }
        last = text;
      }
      std::thread::sleep(std::time::Duration::from_millis(25));
    }
    Err(format!(
      "`{needle}` never appeared in {} within {CEILING:?}; last saw:\n{last}",
      xrs.display()
    ))
  }
}

/// A whole program with `body` as its one statement.
///
/// A helper rather than a literal at each of eight call sites, and plan 64 is
/// why: `print 1` fitted on one line and `println! \-> "#{1}"` does not, so
/// rustfmt wrapped three of them onto their own lines followed by a comma —
/// which is exactly the shape `no_literal_expectations`'s captured-output
/// heuristic counts. The heuristic was right that the shape appeared and
/// wrong about what it meant: these are the test's **inputs**, written to
/// disk as it runs, not an expectation of anything.
fn program(body: &str) -> String {
  format!("main : ()\nmain =\n  {body}\n  ()\n")
}

/// Criterion 1: it builds once at start, then again on a change.
#[test]
fn it_builds_at_start_and_on_change() {
  if support::skip_without_rustc("it_builds_at_start_and_on_change") {
    return;
  }
  let watching = Watching::start("change", &program("println! \\-> \"#{1}\""));

  watching
    .wait_for_output("println!(\"{}\", 1)")
    .expect("the initial build should happen before any change");

  watching.write(&program("println! \\-> \"#{2}\""));
  let after = watching
    .wait_for_output("println!(\"{}\", 2)")
    .expect("a change should trigger a rebuild");
  assert!(
    !after.contains("println!(\"{}\", 1)"),
    "the rebuild should replace the output:\n{after}"
  );
}

/// Criterion 2, and the property most likely to regress: a compile error does
/// not end the loop.
///
/// Plan 12 §1.2 — a watcher that exits on a typo is worse than no watcher,
/// because the user fixes it and nothing happens.
#[test]
fn a_compile_error_does_not_end_the_loop() {
  if support::skip_without_rustc("a_compile_error_does_not_end_the_loop") {
    return;
  }
  let mut watching = Watching::start("survive", &program("println! \\-> \"#{1}\""));
  watching
    .wait_for_output("println!(\"{}\", 1)")
    .expect("the initial build should happen");

  // Break it. The watcher must report and continue.
  watching.write(&program("println! \\-> \"#{(1 +)}\""));
  // Then fix it, with different content so the output is distinguishable.
  watching.write(&program("println! \\-> \"#{3}\""));

  watching
    .wait_for_output("println!(\"{}\", 3)")
    .expect("the loop should still be alive after a compile error");

  // The loop is still alive. That is the criterion, and it is shown by the
  // rebuild above having happened at all — a dead process writes no output —
  // plus the process itself still running.
  assert!(
    watching.child.try_wait().expect("should poll").is_none(),
    "the watcher should still be running after a compile error"
  );
}

/// Criterion 5: the watcher survives the file being **replaced**, which is
/// what editors actually do.
///
/// A watch registered on the file's inode would stop here, and the symptom —
/// "it worked once and then went quiet" — looks like a race rather than a
/// design error. `Target` watches the parent directory precisely so this
/// works, and this test is what holds that decision in place.
#[test]
fn it_survives_the_file_being_replaced() {
  if support::skip_without_rustc("it_survives_the_file_being_replaced") {
    return;
  }
  let watching = Watching::start("replaced", &program("println! \\-> \"#{1}\""));
  watching
    .wait_for_output("println!(\"{}\", 1)")
    .expect("the initial build should happen");

  // Twice, because the failure mode is losing the watch on the *first*
  // replacement: one rename could succeed by luck of event ordering.
  watching.replace(&program("println! \\-> \"#{4}\""));
  watching
    .wait_for_output("println!(\"{}\", 4)")
    .expect("a renamed-over file should still trigger a rebuild");

  watching.replace(&program("println! \\-> \"#{5}\""));
  watching
    .wait_for_output("println!(\"{}\", 5)")
    .expect("and so should the next one");
}

/// Criterion 7: no timing constant decides correctness.
///
/// Asserted by reading the source, which is unusual and is the point: plan 12
/// §1.1's claim is about what the watcher *does not* contain, and the cache is
/// what makes the absence safe. A debounce added later would pass every other
/// test in this file.
#[test]
fn the_watcher_contains_no_debounce() {
  let source = std::fs::read_to_string(
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/watch_command.rs"),
  )
  .expect("should read watch_command.rs");

  let code: String = source
    .lines()
    .filter(|l| !l.trim_start().starts_with("//"))
    .collect::<Vec<_>>()
    .join("\n");

  for heuristic in ["sleep", "Duration", "debounce", "elapsed", "Instant"] {
    assert!(
      !code.contains(heuristic),
      "`{heuristic}` appears in watch_command.rs. Plan 12 §1.1: correctness \
       comes from plan 10's cache, not from timing. If a debounce is genuinely \
       wanted as an optimization, say so in the plan first."
    );
  }
}

/// Criterion 6: the CLI no longer calls `watch` planned.
#[test]
fn the_cli_reports_watch_as_ready() {
  let out = Command::new(env!("CARGO_BIN_EXE_voltaic"))
    .arg("--help")
    .output()
    .expect("should run");
  let help = String::from_utf8_lossy(&out.stdout);

  assert!(help.contains("watch"), "{help}");
  for line in help.lines().filter(|l| l.starts_with("  voltaic watch")) {
    assert!(
      !line.contains("plan 12"),
      "watch is implemented, so --help should not call it planned:\n{line}"
    );
  }

  // And invoking it with no argument asks for one rather than announcing a plan.
  let out = Command::new(env!("CARGO_BIN_EXE_voltaic"))
    .arg("watch")
    .output()
    .expect("should run");
  let stderr = String::from_utf8_lossy(&out.stderr);
  assert!(!stderr.contains("not implemented yet"), "{stderr}");
  assert!(stderr.contains("no input file"), "{stderr}");
}

/// A missing path is refused with a message naming what is accepted, rather
/// than starting a watcher on nothing.
#[test]
fn a_missing_target_is_refused() {
  let out = Command::new(env!("CARGO_BIN_EXE_voltaic"))
    .args(["watch", "/nonexistent/main.sdx"])
    .output()
    .expect("should run");
  assert!(!out.status.success());
  let stderr = String::from_utf8_lossy(&out.stderr);
  assert!(stderr.contains("does not exist"), "{stderr}");
}
