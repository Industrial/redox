# -*- coding: utf-8 -*-
"""Plan 79 batch 1: five of `redox-cli`'s bodies."""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('check_command', '''
export(crate) check_command : &[String] -> Result<(), String>
check_command args =
  -- `parsed`, not a rebinding of `args`: `RDX1003` reports a shadow.
  parsed = (parse_build_args args)?
  -- Bound, because a nested string inside `#{…}` ends the interpolation early.
  shown = .to_string (.display parsed.input)
  source = (.map_err (\\e -> format! \\-> "error: cannot read #{shown}: #{e}") (std::fs::read_to_string (&parsed.input)))?

  outcome = (redox_driver::check (&parsed.input) (&source))?
  report (&outcome) parsed.plain
''')

write('main', '''
main : ExitCode
main =
  args : Vec<String> = std::env::args () |> .skip 1 |> .collect
  case run::run (&args) of
    Ok () -> ExitCode::SUCCESS
    Err message ->
      eprintln! \\-> "#{message}"
      ExitCode::FAILURE
''')

write('check_command/report', '''
export(crate) report : &CheckOutcome -> bool -> Result<(), String>
report outcome plain =
  if .is_empty outcome.diagnostics
    return Ok ()
  rendered = redox_diag::render (&outcome.diagnostics) plain
  if outcome.failed
    return Err rendered
  eprintln! \\-> "#{rendered}"
  Ok ()
''')

write('watch_command/run_build', '''
export(crate) run_build : &str -> bool -> ()
run_build input plain =
  mut args = vec! \\-> (.to_string input)
  if plain
    args |> .push (.to_string "--plain")

  case build_command (&args) of
    Ok _ -> eprintln! \\-> "built #{input}"
    Err message -> eprintln! \\-> "#{message}"
  ()
''')

write('watch_command/report', '''
export(crate) report : &Target -> bool -> ()
report target plain =
  case &target.file of
    Some file ->
      shown = .to_string (.display file)
      run_build (&shown) plain
    -- A directory target compiles every `.sdx` in it, one at a time, because
    -- `build_command` takes one file. Plan 10's cache makes the files that did
    -- not change cost a hash each.
    None ->
      for source in sources (&target.dir)
        run_build (&source) plain
  ()
''')

print('plan 79 batch 1: five files')
