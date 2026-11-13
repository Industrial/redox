# -*- coding: utf-8 -*-
"""Plan 79 batch 2: six of `voltaic-cli`'s sixteen remaining bodies.

Two surface facts this batch leans on, both already recorded elsewhere and both
easy to get wrong from the Rust:

* **An open-ended range has no spelling** — `&args[1..]` is `&args[1 .. n]`
  with `n = .len args` bound first. Plan 68 found it in `walk.sdx`, plan 76 in
  `signature_line.sdx`, and this is its third sighting.
* **`not` is lexed but not parsed as a prefix operator** — `Keyword::Not` has
  no arm in `unary_op.rs`, so negation is `!`, as in `if !.is_empty …`.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('fmt_command/parse_fmt_args', '''
export(crate) parse_fmt_args : &[String] -> Result<FmtArgs, String>
parse_fmt_args args =
  mut inputs = Vec::new ()
  mut check = false
  mut plain = false

  for arg in args
    case .as_str arg of
      "--check" -> check := true
      "--plain" -> plain := true
      -- A guard, where the Rust wrote `flag if flag.starts_with('-')`: the
      -- arm's `|` is doc 03 §6.2's guard and not an alternation, which is
      -- why `or` has that job instead.
      flag | .starts_with '-' flag ->
        help = usage ()
        return Err (format! \\-> "error: unknown flag `#{flag}`\\n\\n#{help}")
      path -> inputs |> .push (PathBuf::from path)

  if .is_empty inputs
    help = usage ()
    return Err (format! \\-> "error: fmt needs at least one .sdx\\n\\n#{help}")

  Ok (FmtArgs { inputs, check, plain })
''')

write('build_command/map_path', '''
export(crate) map_path : &Path -> PathBuf
map_path output =
  mut name = .map_or_else OsString::new OsString::from (.file_name output)
  name |> .push ".map"
  .with_file_name name output

test "appends rather than replacing"
  assert_eq! \\-> map_path (Path::new "main.xrs") \\-> Path::new "main.xrs.map"

test "keeps the directory"
  assert_eq! \\-> map_path (Path::new "/tmp/out/main.xrs") \\-> Path::new "/tmp/out/main.xrs.map"

test "handles a name with no extension"
  assert_eq! \\-> map_path (Path::new "out/main") \\-> Path::new "out/main.map"
''')

write('build_command/default_output', '''
export(crate) default_output : &Path -> PathBuf
default_output input =
  if .is_some_and (\\e -> e == "sdx") (.extension input)
    return .with_extension "xrs" input
  mut name = .to_os_string (.as_os_str input)
  name |> .push ".xrs"
  PathBuf::from name

test "maps sdx to xrs"
  assert_eq! \\-> default_output (Path::new "main.sdx") \\-> PathBuf::from "main.xrs"
  assert_eq! \\-> default_output (Path::new "a/b/main.sdx") \\-> PathBuf::from "a/b/main.xrs"

test "appends to an unexpected extension rather than replacing it"
  assert_eq! \\-> default_output (Path::new "notes.txt") \\-> PathBuf::from "notes.txt.xrs"
''')

write('build_command', '''
export(crate) build_command : &[String] -> Result<(), String>
build_command args =
  -- `parsed`, not a rebinding of `args`: `RDX1003` reports a shadow.
  parsed = (parse_build_args args)?
  shown = .to_string (.display parsed.input)
  source = (.map_err (\\e -> format! \\-> "error: cannot read #{shown}: #{e}") (std::fs::read_to_string (&parsed.input)))?

  program = (.map_err (\\failure -> voltaic_diag::render (&diagnose (&parsed.input) (&source) (&failure)) parsed.plain) (voltaic_parser::parse_source (&source)))?
  -- Doc 03 §12.7's lints — plan 38. Reported and **not** fatal: §12.7 makes
  -- `unwrap` *"lintable rather than prohibited"*, so the warning goes to
  -- stderr and the command carries on.
  --
  -- After the parse, because a warning about a program nobody can compile is
  -- noise, and before the write, so the author sees it with the build they
  -- asked for rather than on the next one.
  warnings = voltaic_driver::lint (&parsed.input) (&source) (Some (&program))
  if !.is_empty warnings
    rendered = voltaic_diag::render (&warnings) parsed.plain
    eprint! \\-> "#{rendered}"

  -- `emitted.0` is the Rust and `emitted.1` the map, where the Rust
  -- destructured `let (rust, map) = …`. Voltaic has no tuple-destructuring
  -- binding, so the pair is bound once and projected.
  emitted = voltaic_emit::emit_with_map (&program) (&source) (&shown)

  out_shown = .to_string (.display parsed.output)
  (.map_err (\\e -> format! \\-> "error: cannot write #{out_shown}: #{e}") (std::fs::write (&parsed.output) (&emitted.0)))?

  -- The sidecar sits beside the `.xrs` under the same stem plus `.map`, so
  -- `main.xrs` has `main.xrs.map`. Plan 07 finds it by that rule rather than
  -- by being told where it is.
  --
  -- `sidecar`, where the Rust named the local `map_path`: that spelling
  -- shadows the function of the same name, and `RDX1003` reports it.
  sidecar = map_path (&parsed.output)
  sidecar_shown = .to_string (.display sidecar)
  (.map_err (\\e -> format! \\-> "error: cannot write #{sidecar_shown}: #{e}") (std::fs::write (&sidecar) (voltaic_diag::render_map (&emitted.1))))?
  Ok ()
''')

write('fmt_command', '''
export(crate) fmt_command : &[String] -> Result<(), String>
fmt_command args =
  parsed = (parse_fmt_args args)?
  mut unformatted : Vec<String> = Vec::new ()

  for path in &parsed.inputs
    shown = .to_string (.display path)
    source = (.map_err (\\e -> format! \\-> "error: cannot read #{shown}: #{e}") (std::fs::read_to_string path))?

    program = (.map_err (\\failure -> voltaic_diag::render (&diagnose path (&source) (&failure)) parsed.plain) (voltaic_parser::parse_source (&source)))?

    formatted = voltaic_fmt::format (&program) (&source)
    if formatted == source
      continue

    if parsed.check
      unformatted |> .push shown
    else
      (.map_err (\\e -> format! \\-> "error: cannot write #{shown}: #{e}") (std::fs::write path (&formatted)))?

  if .is_empty unformatted
    return Ok ()
  indented : Vec<String> = unformatted |> .iter |> .map (\\p -> format! \\-> "  #{p}") |> .collect
  listed = .join "\\n" indented
  Err (format! \\-> "error: not formatted:\\n#{listed}\\nrun `voltaic fmt` on them\\n")
''')

write('run', '''
export(crate) run : &[String] -> Result<(), String>
run args =
  guard .map String::as_str (.first args) is Some first else
    help = usage ()
    print! \\-> "#{help}"
    return Ok ()

  guard lookup first is Some command else
    help = usage ()
    return Err (format! \\-> "error: unknown command `#{first}`\\n\\n#{help}")

  -- A declared command whose plan has not landed says so, and names the plan.
  if .not_yet command is Some message
    return Err message

  -- `args[1 .. n]` with `n` bound first, where the Rust wrote `&args[1..]`.
  -- An **open-ended range has no spelling** — plan 68's gap, third sighting.
  n = .len args
  rest = &args[1 .. n]

  case command.name of
    "--help" ->
      help = usage ()
      print! \\-> "#{help}"
      Ok ()
    "--version" ->
      version = env! \\-> "CARGO_PKG_VERSION"
      println! \\-> "voltaic #{version}"
      Ok ()
    "build" -> build_command rest
    "check" -> check_command rest
    "fmt" -> fmt_command rest
    "watch" -> watch_command rest
    -- Unreachable while `COMMANDS` and this case agree, and
    -- `cli_surface.rs::every_ready_command_dispatches` is what keeps them
    -- agreeing. A panic rather than a silent success: a `Ready` command with
    -- no arm is a bug in Voltaic, not in the invocation.
    other ->
      unreachable! \\-> "`#{other}` is Ready in COMMANDS but has no dispatch arm"
''')

print('plan 79 batch 2: six files')
