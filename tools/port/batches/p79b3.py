# -*- coding: utf-8 -*-
"""Plan 79 batch 3: four more of `redox-cli`'s bodies, and the watcher.

`watch_command` is the first ported body that needs a **move closure whose
Rust discarded a `#[must_use]`**. Plan 75 §2.3 established `_dropped = …` as
the spelling for `let _ = …`, and it does not reach here: a lambda body is one
expression, so there is nowhere to put a binding. `drop (…)` is the answer, and
it says the same thing.

`sources`'s test writes a `.sdx` fixture as a string, which is the one place
`\\#` earns its keep: `\\#{1}` is a literal `#{1}` and not an interpolation.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('run/commands/lookup', '''
export(crate) lookup : &str -> Option<&'static Command>
lookup arg = .find (\\c -> .matches arg c) (.iter COMMANDS)

test "every declared command is reachable by its own name"
  for command in COMMANDS
    found = .unwrap_or_else (\\-> panic! \\-> "#{command.name} not found") (lookup command.name)
    assert_eq! \\-> found.name \\-> command.name

test "an unknown argument selects nothing"
  assert! \\-> .is_none (lookup "frobnicate")
  assert! \\-> .is_none (lookup "")

-- | Two entries matching the same argument would make dispatch depend on
-- | table order, which is the kind of thing nobody notices until the order
-- | changes.
test "no two commands claim the same argument"
  for arg in ["build", "check", "fmt", "watch", "--version", "-V", "--help", "-h", "help"]
    claimants = .count (.filter (\\c -> .matches arg c) (.iter COMMANDS))
    assert_eq! \\-> claimants \\-> 1 \\-> "#{arg} is claimed #{claimants} times"
''')

write('watch_command/sources', '''
export(crate) sources : &Path -> Vec<String>
sources dir =
  guard std::fs::read_dir dir is Ok entries else
    return Vec::new ()
  mut found : Vec<String> = entries
    |> .filter_map Result::ok
    |> .map (\\e -> .path e)
    |> .filter (\\p -> .is_file p && .is_some_and (\\x -> x == "sdx") (.extension p))
    |> .map (\\p -> .to_string (.display p))
    |> .collect
  .sort found
  found

test "finds and sorts sdx files"
  pid = std::process::id ()
  dir = .join (format! \\-> "redox-watch-src-#{pid}") (std::env::temp_dir ())
  .expect "should create" (std::fs::create_dir_all (&dir))
  -- `\\#{1}` is a literal `#{1}`, so the fixture is Redox source rather than
  -- this file interpolating into it.
  for name in ["b.sdx", "a.sdx", "ignored.rs"]
    .expect "should write" (std::fs::write (.join name dir) "main : ()\\nmain =\\n  println! \\\\-> \\"\\#{1}\\"\\n")

  found = sources (&dir)
  assert_eq! \\-> .len found \\-> 2 \\-> "#{found:?}"
  assert! \\-> .ends_with "a.sdx" found[0] && .ends_with "b.sdx" found[1] \\-> "#{found:?}"

  _dropped = .ok (std::fs::remove_dir_all (&dir))

test "an unreadable directory yields nothing"
  found = sources (std::path::Path::new "/nonexistent/redox")
  assert! \\-> .is_empty found
''')

write('watch_command', '''
export(crate) watch_command : &[String] -> Result<(), String>
watch_command args =
  input = (.ok_or_else (\\-> .to_string "error: no input file\\n\\n`redox watch` takes a .sdx file or a directory.") (.find (\\a -> !.starts_with '-' a) (.iter args)))?
  target = (Target::resolve (Path::new input))?
  plain = .any (\\a -> a == "--plain") (.iter args)

  -- Build before watching, so an already-broken file is reported now rather
  -- than after the user's next keystroke.
  report (&target) plain

  -- `tx` and `rx` bound from the pair, where the Rust destructured
  -- `let (tx, rx) = …`: Redox has no tuple-destructuring binding, so each
  -- half is moved out by projection.
  channel = mpsc::channel ()
  tx = channel.0
  rx = channel.1
  -- A send failure means the receiver is gone, which means the process is
  -- ending; there is nothing useful to do about it here.
  --
  -- `drop (…)`, where the Rust wrote `let _ = …`. A lambda body is one
  -- expression, so plan 75 §2.3's `_dropped = …` binding has nowhere to go.
  mut watcher = (.map_err (\\e -> format! \\-> "error: cannot start the file watcher: #{e}") (notify::recommended_watcher (move \\event -> drop (.send event tx))))?

  dir_shown = .to_string (.display target.dir)
  watched = watcher |> .watch (&target.dir) RecursiveMode::Recursive
  (.map_err (\\e -> format! \\-> "error: cannot watch #{dir_shown}: #{e}") watched)?

  eprintln! \\-> "watching #{dir_shown} — press ctrl-c to stop"

  for received in rx
    case received of
      Ok event ->
        -- Two filters, and both are necessary. `is_change` rejects the
        -- build's own *read* of the source, which is a `.sdx` path and would
        -- otherwise retrigger the build forever (plan 12 §10.1 — measured at
        -- 97,442 builds in six seconds). `is_relevant` rejects the generated
        -- `.xrs` it writes, which is a real change to an irrelevant path.
        if is_change (&event.kind) && .any (\\p -> is_relevant p) (.iter event.paths)
          report (&target) plain
      -- A watcher error — a deleted directory, an exhausted inotify watch
      -- limit — is reported and survived, for the same reason a compile error
      -- is: exiting means the user fixes the cause and nothing happens.
      Err e -> eprintln! \\-> "error: the file watcher reported: #{e}"

  Ok ()
''')

write('build_command/parse_build_args', '''
export(crate) parse_build_args : &[String] -> Result<BuildArgs, String>
parse_build_args args =
  mut input : Option<PathBuf> = None
  mut output : Option<PathBuf> = None
  mut plain = false
  mut i = 0
  while i < .len args
    case .as_str args[i] of
      -- `or`, not `|`: an arm's `|` is doc 03 §6.2's guard, so alternation
      -- is the word.
      "-o" or "--output" ->
        path = (.ok_or_else (\\-> .to_string "error: -o requires a path") (.get (i + 1) args))?
        output := Some (PathBuf::from path)
        i += 2
      "--plain" ->
        plain := true
        i += 1
      flag | .starts_with '-' flag ->
        help = usage ()
        return Err (format! \\-> "error: unknown flag `#{flag}`\\n\\n#{help}")
      path ->
        if .is_some input
          return Err (format! \\-> "error: unexpected second input `#{path}`")
        input := Some (PathBuf::from path)
        i += 1
  -- `resolved` and `out`, where the Rust rebound `input` and `output`:
  -- `RDX1003` reports a shadow. A `guard` rather than `ok_or_else`, because
  -- the message needs `usage ()` bound and a lambda body is one expression.
  guard input is Some resolved else
    help = usage ()
    return Err (format! \\-> "error: no input file\\n\\n#{help}")
  out = .unwrap_or_else (\\-> default_output (&resolved)) output
  Ok (BuildArgs { input = resolved, output = out, plain })

test "rejects an unknown flag"
  args = [.to_string "--nope"]
  err = .expect_err "should fail" (parse_build_args (&args))
  assert! \\-> .contains "unknown flag" err \\-> "#{err}"

test "rejects a missing input"
  empty : Vec<String> = Vec::new ()
  err = .expect_err "should fail" (parse_build_args (&empty))
  assert! \\-> .contains "no input file" err \\-> "#{err}"

test "rejects a second input"
  args = [.to_string "a.sdx", .to_string "b.sdx"]
  err = .expect_err "should fail" (parse_build_args (&args))
  assert! \\-> .contains "second input" err \\-> "#{err}"

test "rejects a dangling output flag"
  args = [.to_string "-o"]
  err = .expect_err "should fail" (parse_build_args (&args))
  assert! \\-> .contains "requires a path" err \\-> "#{err}"

-- | Plan 05 leaf 5: the UI corpus needs a flag, not an environment sniff.
test "accepts the plain flag before or after the input"
  before = [.to_string "--plain", .to_string "a.sdx"]
  after = [.to_string "a.sdx", .to_string "--plain"]
  for args in [before, after]
    parsed = .expect "should parse" (parse_build_args (&args))
    assert! \\-> parsed.plain
    assert_eq! \\-> .to_str parsed.input \\-> Some "a.sdx"

test "defaults to graphical rendering"
  args = [.to_string "a.sdx"]
  parsed = .expect "should parse" (parse_build_args (&args))
  assert! \\-> !parsed.plain
''')

print('plan 79 batch 3: four files')
