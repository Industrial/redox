# -*- coding: utf-8 -*-
"""Plan 77 batch 2: `build` and `source_map`.

Both are the shape this crate repeats — read, project, answer — and both build
a sidecar path by pushing onto an `OsString`, which is the one thing in them
that does not fit a pipeline: `push` returns `()`, so the value has to be bound
and mutated.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('build', '''
export build : &BuildRequest -> Result<BuildResponse, String>
build request =
  path = std::path::Path::new (&request.path)
  shown = .clone request.path
  source = (.map_err (\\e -> format! \\-> "cannot read #{shown}: #{e}") (std::fs::read_to_string path))?

  -- A syntax error is Voltaic's own and never reaches the emitter. Returned as a
  -- diagnostic rather than an `Err`, because a program that does not parse is
  -- an answer about the program (plan 50 §5 criterion 2).
  program = case voltaic_parser::parse_source (&source) of
    Ok parsed -> parsed
    Err failure ->
      return Ok (BuildResponse {
        ok = false,
        output = None,
        map = None,
        diagnostics = diagnose path (&source) (&failure)
          |> .iter
          |> .map diagnostic_from_voltaic
          |> .collect,
      })

  diagnostics = voltaic_driver::lint path (&source) (Some (&program))
  emitted = voltaic_emit::emit_with_map (&program) (&source) (&request.path)
  -- `generated`, not `rust`: **`rust` is a keyword**, and as a binding name it
  -- earns `RDX0101: expected a """ block of Rust after `rust`` — a diagnostic
  -- about the escape hatch, which is not what the author wrote. Three keyword
  -- collisions in three crates now, and a message naming the collision would
  -- have cost a minute instead of a puzzle.
  generated = emitted.0
  map = emitted.1

  output = .with_extension "xrs" path
  out_shown = .to_string (.display output)
  (.map_err (\\e -> format! \\-> "cannot write #{out_shown}: #{e}") (std::fs::write (&output) (&generated)))?

  -- Same rule as `voltaic build`: the stem plus `.map`, so `main.xrs` has
  -- `main.xrs.map` rather than `main.map`.
  --
  -- Bound and mutated because `OsString::push` returns `()`, so there is no
  -- value for a pipeline to carry.
  mut map_name = .to_os_string (.expect "a name" (.file_name output))
  .push ".map" map_name
  map_path = .with_file_name map_name output
  map_shown = .to_string (.display map_path)
  (.map_err (\\e -> format! \\-> "cannot write #{map_shown}: #{e}") (std::fs::write (&map_path) (voltaic_diag::render_map (&map))))?

  Ok (BuildResponse {
    ok = true,
    output = Some out_shown,
    map = Some map_shown,
    diagnostics = diagnostics |> .iter |> .map diagnostic_from_voltaic |> .collect,
  })
''')

write('source_map', '''
export source_map : &SourceMapRequest -> Result<SourceMapResponse, String>
source_map request =
  xrs = std::path::Path::new (&request.path)
  shown = .clone request.path
  -- The sidecar's name is the full file name plus `.map`, so `main.xrs` has
  -- `main.xrs.map`. Accepting either the `.xrs` or the `.map` itself, because
  -- a caller holding a stack trace has the `.xrs` and a caller that just ran
  -- `voltaic_build` has both paths from its response.
  is_map = .is_some_and (\\e -> e == "map") (.extension xrs)
  map_path = if is_map
    .to_path_buf xrs
  else
    named = .ok_or_else (\\-> format! \\-> "#{shown} names no file") (.file_name xrs)
    mut name = .to_os_string named?
    .push ".map" name
    .with_file_name name xrs

  map_shown = .to_string (.display map_path)
  source_text = .map_err (\\e -> format! \\-> "cannot read #{map_shown}: #{e}. `voltaic_build` writes it beside the .xrs") (std::fs::read_to_string (&map_path))
  map = (.map_err (\\e -> format! \\-> "#{map_shown} is not a source map: #{e}") (voltaic_diag::parse_map (&source_text?)))?

  column = .unwrap_or 1 request.column
  case .resolve request.line column map of
    Some at ->
      Ok (SourceMapResponse {
        source = Some (.clone map.source_path),
        line = Some at.0,
        column = Some at.1,
        reason = None,
      })
    None ->
      reason = if request.line == 0 || request.line > .len map
        "out-of-range: the .xrs has no such line"
      else
        "synthetic: Voltaic emitted this line and no .sdx line is behind it"
      Ok (SourceMapResponse {
        source = Some (.clone map.source_path),
        line = None,
        column = None,
        reason = Some (.to_string reason),
      })
''')

print('plan 77 batch 2: two files')
