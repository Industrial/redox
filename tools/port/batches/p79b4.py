# -*- coding: utf-8 -*-
"""Plan 79 batch 4: `redox-cli`'s last six bodies.

Three shapes this batch meets for the first time in a port:

* **`matches!` with `|` alternation.** An arm's `|` is doc 03 §6.2's *guard*,
  so the macro does not translate as written. `or` says alternation, and here
  three `if`s say it flatter than a `case` nested inside a `case`.
* **A module-scope `static`.** `Dir::new`'s counter is a `static` inside the
  Rust `fn`; Redox's `const`/`static` is module-scope (doc 03 §13.4), so it is
  hoisted and `#[cfg(test)]`-gated with the helper it belongs to.
* **A test-scoped helper *type*.** Redox puts nothing but `#[test] fn`s inside
  its generated `mod __rdx_tests` — plan 73 §2.9 — so `Dir`, its `static` and
  its `Drop` live at file scope behind `#[cfg(test)]`.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('run/command', '''
instance Command
  -- | The argument forms that select this command.
  -- |
  -- | `--help` is also `-h` and `help`, and `--version` is also `-V`, because
  -- | those are the conventions every other tool follows and a surface this
  -- | small gains nothing by being surprising.
  -- |
  -- | Three `if`s, where the Rust nested `matches!` inside a `match`: an
  -- | arm's `|` is doc 03 §6.2's guard, so the macro does not carry over as
  -- | written, and `or` inside a `case` inside a `case` says it twice as
  -- | deep as this does.
  export(crate) matches : &str -> &self -> bool
  matches arg self =
    if self.name == "--help"
      return arg == "--help" || arg == "-h" || arg == "help"
    if self.name == "--version"
      return arg == "--version" || arg == "-V"
    arg == self.name

  -- | The message for a command that is declared but not yet implemented.
  export(crate) not_yet : &self -> Option<String>
  not_yet self = case self.state of
    State::Ready -> None
    State::Planned { plan, id } ->
      Some (format! \\-> "error: `#{self.name}` is not implemented yet — it is plan #{plan:02} (`#{id}`)")

-- | `#[cfg(test)]`: the two commands every test below reads. Redox can put
-- | nothing but `#[test] fn`s inside its generated `mod __rdx_tests` —
-- | plan 73 §2.9 — so a test's `const` lives at file scope.
#[cfg(test)]
const FMT : Command = Command { name = "fmt", summary = "format a .sdx in place", state = State::Planned { plan = 34, id = "formatter" } }

#[cfg(test)]
const BUILD : Command = Command { name = "build", summary = "transpile", state = State::Ready }

test "a ready command has nothing to announce"
  assert_eq! \\-> .not_yet BUILD \\-> None

test "a planned command names its plan"
  message = .expect "fmt is planned" (.not_yet FMT)
  assert! \\-> .contains "not implemented yet" message \\-> "#{message}"
  assert! \\-> .contains "plan 34" message \\-> "#{message}"
  assert! \\-> .contains "formatter" message \\-> "#{message}"

test "the help and version aliases are accepted"
  help = Command { name = "--help", summary = "", state = State::Ready }
  for arg in ["--help", "-h", "help"]
    assert! \\-> .matches arg help \\-> "#{arg}"
  assert! \\-> !.matches "--helpful" help

  version = Command { name = "--version", summary = "", state = State::Ready }
  assert! \\-> .matches "--version" version && .matches "-V" version
  assert! \\-> !.matches "-v" version \\-> "lowercase -v is not a version flag"

test "an ordinary command matches only itself"
  assert! \\-> .matches "build" BUILD
  assert! \\-> !.matches "builds" BUILD && !.matches "--build" BUILD
''')

write('run/commands', '''
-- `fmt`'s summary is plural, because plan 34 §1.5's gate is
-- `redox fmt --check examples/*.sdx` and a one-file command would make that a
-- shell loop nobody checks. The comment sat inside the table in the Rust; the
-- table is one line here, so it sits above it.
export(crate) const COMMANDS : &[Command] = &[Command { name = "build", summary = "transpile a .sdx to .xrs", state = State::Ready }, Command { name = "check", summary = "transpile, then report rustc's errors against the .sdx", state = State::Ready }, Command { name = "fmt", summary = "format .sdx files in place, or --check them", state = State::Ready }, Command { name = "watch", summary = "rebuild on change", state = State::Ready }, Command { name = "--version", summary = "print the version", state = State::Ready }, Command { name = "--help", summary = "print this message", state = State::Ready }]

test "every command has a summary"
  for command in COMMANDS
    assert! \\-> !.is_empty command.summary \\-> "#{command.name} has no summary"

-- | Doc 01 §27 fixes the surface at six entries. This is the table's own
-- | count; `cli_surface.rs` holds the list, deliberately duplicated.
test "the surface has six entries"
  assert_eq! \\-> .len COMMANDS \\-> 6 \\-> "doc 01 §27 lists six commands; adding a seventh is a decision, not a patch"
''')

write('run/usage', '''
export(crate) usage : String
usage =
  mut out = String::from "redox — a MoonScript-inspired surface for Rust\\n\\nusage:\\n"

  width = COMMANDS |> .iter |> .map (\\c -> .len c.name) |> .max |> .unwrap_or 0
  for command in COMMANDS
    name = command.name
    _dropped = write! \\-> out \\-> "  redox #{name:<width$}  #{command.summary}"
    case command.state of
      State::Planned { plan, id } ->
        _planned = write! \\-> out \\-> " (plan #{plan:02}, `#{id}`)"
      State::Ready -> ()
    out |> .push '\\n'

  out |> .push_str "\\nCargo owns build, test, docs and dependencies (doc 01 §27).\\n"
  out

-- | Criterion 2: a declared command cannot be hidden from `--help`.
test "every command appears"
  text = usage ()
  for command in COMMANDS
    assert! \\-> .contains command.name text \\-> "#{command.name} is missing from the usage text:\\n#{text}"
    assert! \\-> .contains command.summary text \\-> "#{command.name} summary missing"

-- | **No command advertises a plan**, because every declared command is now
-- | built.
-- |
-- | This asserted the opposite until plan 34 — it required `fmt` to say
-- | *"plan 34"* — and its own comment recorded the pattern: *"`watch` was
-- | listed here until plan 12 implemented it, and this test failed when it
-- | did, which is the usage text and the table staying in step."* `fmt` was
-- | the last planned command, so the assertion inverts rather than moving to
-- | another name.
-- |
-- | Plan 11 §2.1's guarantee is unchanged and still enforced: a `Ready`
-- | command must not advertise a plan, and now none may, because none is
-- | planned. A future row that declares a command ahead of itself makes this
-- | fail and has to say so.
test "no command advertises a plan"
  text = usage ()
  assert! \\-> !.contains "plan " text \\-> "every command is implemented, so none should name a plan:\\n#{text}"

-- | The usage text is one screen. A tool whose help needs scrolling has a
-- | surface problem, so the length is an assertion rather than a hope.
test "fits on one screen"
  text = usage ()
  lines = .count (.lines text)
  assert! \\-> lines <= 24 \\-> "usage is #{lines} lines, which no longer fits a terminal"

test "names cargo as the owner of everything else"
  text = usage ()
  assert! \\-> .contains "Cargo owns" text
''')

write('watch_command/is_relevant', '''
export(crate) is_relevant : &Path -> bool
is_relevant path =
  guard .map (\\n -> .to_string (.to_string_lossy n)) (.file_name path) is Some name else
    return false

  -- Vim writes `4913` to test whether a directory is writable, and leaves
  -- `.swp`/`.swx` files; Emacs uses `#name#` and `.#name`; many editors and
  -- tools use a `~` suffix or a `.tmp` extension.
  if name == "4913" || .ends_with '~' name || .starts_with ".#" name || (.starts_with '#' name && .ends_with '#' name)
    return false

  extension = .map (\\e -> .to_string (.to_string_lossy e)) (.extension path)
  case .as_deref extension of
    Some "sdx" -> !.starts_with '.' name
    _ -> false

test "a sdx file is relevant"
  assert! \\-> is_relevant (Path::new "src/main.sdx")
  assert! \\-> is_relevant (Path::new "main.sdx")

test "another extension is not"
  for name in ["main.rs", "main.xrs", "Cargo.toml", "README.md", "main"]
    assert! \\-> !is_relevant (Path::new name) \\-> "#{name}"

-- | `.xrs` is Redox's *output*. Compiling because the output changed would
-- | be a loop that feeds itself.
test "the generated output is not relevant"
  assert! \\-> !is_relevant (Path::new "main.xrs")
  assert! \\-> !is_relevant (Path::new "main.xrs.map")

test "editor temporaries are ignored"
  -- `4913` is vim's writability probe, `main.sdx~` a backup suffix,
  -- `.main.sdx.swp` vim's swap file, `.\\#main.sdx` emacs' lock file,
  -- `\\#main.sdx\\#` emacs' auto-save, and `.main.sdx` a dotfile shadowing the
  -- source. The Rust spelled each one in a trailing comment inside the array;
  -- the array is one line here, so they are listed above it.
  for name in ["4913", "main.sdx~", ".main.sdx.swp", ".\\#main.sdx", "\\#main.sdx\\#", ".main.sdx"]
    assert! \\-> !is_relevant (Path::new name) \\-> "#{name} should be ignored"

test "a path with no file name is not relevant"
  assert! \\-> !is_relevant (Path::new "/")
  assert! \\-> !is_relevant (Path::new "..")

-- | A deny-list errs toward compiling. Asserting that explicitly, because the
-- | opposite default — an allow-list — would silently stop rebuilding for an
-- | editor nobody tested.
test "an unfamiliar name still compiles if it is a sdx"
  assert! \\-> is_relevant (Path::new "weird-editor-artifact.sdx")
''')

write('watch_command/is_change', '''
export(crate) is_change : &EventKind -> bool
is_change kind = case kind of
  EventKind::Create _ or EventKind::Remove _ -> true
  -- An unrecognised kind is treated as a change: a watcher that misses an
  -- edit looks broken, and the cache does make the redundant compile cheap
  -- *provided* it is not self-triggering. Every self-triggering kind is
  -- named below and excluded.
  EventKind::Any or EventKind::Other -> true
  -- Nested, where the Rust alternated inside the constructor
  -- (`Modify(Data(_) | Name(_))`). An arm's `|` is a guard, so the
  -- alternation moves one level in rather than staying a pattern.
  EventKind::Modify inner -> case inner of
    ModifyKind::Data _ or ModifyKind::Name _ -> true
    _ -> false
  EventKind::Access _ -> false

-- | The event that caused the loop. If this ever returns `true` again, the
-- | watcher will spin on its own reads.
test "reading a file is not a change"
  assert! \\-> !is_change (&EventKind::Access (notify::event::AccessKind::Open notify::event::AccessMode::Any))
  assert! \\-> !is_change (&EventKind::Access notify::event::AccessKind::Read)
  assert! \\-> !is_change (&EventKind::Access (notify::event::AccessKind::Close notify::event::AccessMode::Read))
  assert! \\-> !is_change (&EventKind::Access notify::event::AccessKind::Any)

-- | Writing is. `Close(Write)` is an `Access` kind and still excluded — the
-- | `Modify(Data)` event that accompanies the same write is what triggers.
test "writing data is a change"
  assert! \\-> is_change (&EventKind::Modify (ModifyKind::Data notify::event::DataChange::Any))
  assert! \\-> is_change (&EventKind::Modify (ModifyKind::Data notify::event::DataChange::Content))

-- | A rename over the source is how editors save, and it is a change.
test "renaming is a change"
  assert! \\-> is_change (&EventKind::Modify (ModifyKind::Name notify::event::RenameMode::To))
  assert! \\-> is_change (&EventKind::Modify (ModifyKind::Name notify::event::RenameMode::Both))

test "creating and removing are changes"
  assert! \\-> is_change (&EventKind::Create notify::event::CreateKind::File)
  assert! \\-> is_change (&EventKind::Remove notify::event::RemoveKind::File)

-- | Metadata is excluded, and the reason is not fastidiousness: under
-- | `relatime` the build's own read can update `atime`, which would reopen
-- | the loop by another door.
test "metadata alone is not a change"
  assert! \\-> !is_change (&EventKind::Modify (ModifyKind::Metadata notify::event::MetadataKind::Any))
  assert! \\-> !is_change (&EventKind::Modify (ModifyKind::Metadata notify::event::MetadataKind::AccessTime))

-- | An unknown kind errs toward rebuilding, because a missed edit looks like
-- | a broken watcher.
test "an unknown kind is treated as a change"
  assert! \\-> is_change (&EventKind::Any)
  assert! \\-> is_change (&EventKind::Other)
''')

write('watch_command/target', '''
instance Target
  -- | Resolve `arg` to a watch target.
  export(crate) resolve : &Path -> Result<Self, String>
  resolve arg =
    if .is_dir arg
      return Ok (Target { dir = .to_path_buf arg, file = None })

    if !.exists arg
      shown = .to_string (.display arg)
      return Err (format! \\-> "error: #{shown} does not exist\\n\\n`redox watch` takes a .sdx file or a directory of them.")

    parent = .filter (\\p -> !.is_empty (.as_os_str p)) (.parent arg)
    -- A bare `main.sdx` has a parent of `""`, which is not a directory any
    -- watcher will accept; `.` is the same place and is.
    dir = .to_path_buf (.unwrap_or (Path::new ".") parent)
    Ok (Target { dir, file = Some (.to_path_buf arg) })

-- | `#[cfg(test)]`: a unique temporary directory, removed when it goes out of
-- | scope. Redox can put nothing but `#[test] fn`s inside its generated
-- | `mod __rdx_tests` — plan 73 §2.9 — so the helper type, its counter and
-- | its `Drop` all live here at file scope.
-- |
-- | The counter is a module-scope `static` where the Rust declared it inside
-- | `Dir::new`: doc 03 §13.4's `const`/`static` is module-scope, and the
-- | `#[cfg(test)]` is what keeps it out of a release build.
#[cfg(test)]
newtype Dir = Dir PathBuf

#[cfg(test)]
static NEXT : std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new 0

#[cfg(test)]
instance Dir
  new : &str -> Self
  new tag =
    n = .fetch_add 1 std::sync::atomic::Ordering::Relaxed NEXT
    pid = std::process::id ()
    dir = .join (format! \\-> "redox-target-#{tag}-#{pid}-#{n}") (std::env::temp_dir ())
    .expect "should create" (std::fs::create_dir_all (&dir))
    Dir dir

#[cfg(test)]
instance Drop Dir
  drop : &mut self -> ()
  drop self =
    _dropped = .ok (std::fs::remove_dir_all (&self.0))
    ()

test "a directory is watched whole"
  dir = Dir::new "whole"
  target = .expect "should resolve" (Target::resolve (&dir.0))
  assert_eq! \\-> target.dir \\-> dir.0
  assert_eq! \\-> target.file \\-> None \\-> "a directory compiles all of itself"

-- | The property that keeps the watcher alive across a save: the *parent* is
-- | registered, not the file.
test "a file is watched through its parent"
  dir = Dir::new "parent"
  file = .join "main.sdx" dir.0
  .expect "should write" (std::fs::write (&file) "main : ()\\nmain =\\n  println! \\\\-> \\"\\#{1}\\"\\n")

  target = .expect "should resolve" (Target::resolve (&file))
  assert_eq! \\-> target.dir \\-> dir.0 \\-> "the parent directory is what is watched"
  assert_eq! \\-> target.file \\-> Some file

test "a missing path is an error that says what is accepted"
  err = .expect_err "should fail" (Target::resolve (Path::new "/nonexistent/main.sdx"))
  assert! \\-> .contains "does not exist" err \\-> "#{err}"
  assert! \\-> .contains ".sdx file or a directory" err \\-> "#{err}"

-- | A bare filename's parent is `""`, which no watcher accepts.
test "a bare filename resolves to the current directory"
  dir = Dir::new "bare"
  file = .join "main.sdx" dir.0
  .expect "should write" (std::fs::write (&file) "main : ()\\nmain =\\n  println! \\\\-> \\"\\#{1}\\"\\n")

  previous = .expect "a cwd" (std::env::current_dir ())
  .expect "should chdir" (std::env::set_current_dir (&dir.0))
  target = .expect "should resolve" (Target::resolve (Path::new "main.sdx"))
  .expect "should restore" (std::env::set_current_dir previous)

  assert_eq! \\-> target.dir \\-> Path::new "." \\-> "not the empty path"
''')

print('plan 79 batch 4: six files')
