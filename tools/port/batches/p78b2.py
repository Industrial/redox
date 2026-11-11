# -*- coding: utf-8 -*-
"""Plan 78 batch 2: `symbols`, `publish`, `convert`, `formatting`,
`diagnostics` and `main`.

Nothing new in the surface. Two shapes already paid for recur: a let-else is a
`case` whose arm returns, and `vec![x]` is `vec! \\-> x`.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('serve/symbols', '''
export(crate) symbols : &str -> DocumentSymbolResponse
symbols source =
  program = case redox_parser::parse_source source of
    Ok parsed -> parsed
    Err _ ->
      return DocumentSymbolResponse::Nested (Vec::new ())
  found : Vec<DocumentSymbol> = program.items
    |> .iter
    |> .map (\\item -> symbol_of item source)
    |> .collect
  DocumentSymbolResponse::Nested found
''')

write('serve/publish', '''
export(crate) publish : &Connection -> &Uri -> &Documents -> Result<(), String>
publish connection uri documents =
  source = .unwrap_or_default (.text uri documents)
  params = PublishDiagnosticsParams {
    uri = .clone uri,
    diagnostics = diagnostics uri source,
    version = None,
  }
  encoded = (.map_err (\\e -> .to_string e) (serde_json::to_value params))?
  message = Message::Notification (Notification {
    method = .to_string PublishDiagnostics::METHOD,
    params = encoded,
  })
  .map_err (\\e -> .to_string e) (.send message connection.sender)
''')

write('serve/diagnostics/convert', '''
export(crate) convert : &redox_diag::Diagnostic -> &str -> Diagnostic
convert diagnostic source =
  message = case &diagnostic.help of
    Some help -> format! \\-> "#{diagnostic.message}\\nhelp: #{help}"
    None -> .clone diagnostic.message
  -- `is_fatal` rather than a match on the variant: plan 38 made `Severity`
  -- the thing that knows, and an integer in a protocol is exactly where an
  -- off-by-one would be invisible.
  severity = if .is_fatal diagnostic.severity
    DiagnosticSeverity::ERROR
  else
    DiagnosticSeverity::WARNING
  Diagnostic {
    range = range_of source diagnostic.span,
    severity = Some severity,
    code = Some (NumberOrString::String (.text diagnostic.code)),
    code_description = None,
    source = Some (.to_string "redox"),
    message,
    related_information = None,
    tags = None,
    -- `r#data`, because **`data` is a keyword** and `lsp_types` named the
    -- field `data`. The raw form is the same identifier in Rust, so nothing
    -- downstream changes. Fourth keyword collision in four crates, and the
    -- first on a **record-literal field** rather than a parameter or binding.
    r#data = None,
  }
''')

write('serve/formatting', '''
export(crate) formatting : &str -> Option<Vec<TextEdit>>
formatting source =
  program = (.ok (redox_parser::parse_source source))?
  formatted = redox_fmt::format (&program) source
  if formatted == source
    return Some (Vec::new ())
  edit = TextEdit {
    range = Range {
      start = Position {
        line = 0,
        character = 0,
      },
      end = to_position source (.len source),
    },
    new_text = formatted,
  }
  Some (vec! \\-> edit)
''')

write('serve/diagnostics', '''
export(crate) diagnostics : &Uri -> &str -> Vec<Diagnostic>
diagnostics uri source =
  path = path_of uri
  mut found : Vec<Diagnostic> = Vec::new ()

  parsed = case redox_parser::parse_source source of
    Ok program -> Some program
    Err failure ->
      for diagnostic in redox_driver::diagnose (&path) source (&failure)
        found |> .push (convert (&diagnostic) source)
      None
  for diagnostic in redox_driver::lint (&path) source (.as_ref parsed)
    found |> .push (convert (&diagnostic) source)
  found
''')

write('main', '''
main : Result<(), String>
main =
  -- Before the transport: `Connection::stdio()` takes the handles and the
  -- handshake blocks, so a flag checked after it would never be reached.
  wants_version = std::env::args ()
    |> .skip 1
    |> .any (\\arg -> arg == "--version" || arg == "-V")
  if wants_version
    version = env! \\-> "CARGO_PKG_VERSION"
    println! \\-> "redox-lsp #{version}"
    return Ok ()

  stdio = Connection::stdio ()
  connection = stdio.0
  threads = stdio.1

  -- The handshake has to come before the loop: an editor sends no request
  -- until it has the capability list, so a server that started serving first
  -- would wait forever on a connection the editor is also waiting on.
  advertised = (.map_err (\\e -> .to_string e) (serde_json::to_value (redox_lsp::capabilities ())))?
  params = (.map_err (\\e -> .to_string e) (.initialize advertised connection))?
  -- Deserialised and dropped. Nothing here needs a workspace root — every
  -- capability answers from the document the editor sent — and parsing it
  -- anyway means a malformed `initialize` fails here rather than somewhere
  -- later and stranger.
  _settings : InitializeParams = (.map_err (\\e -> .to_string e) (serde_json::from_value params))?

  (redox_lsp::serve (&connection))?
  -- The join matters: `lsp-server`'s threads own the stdio handles, and
  -- returning without it can drop a reply the editor is still reading.
  .map_err (\\e -> .to_string e) (.join threads)
''')

print('plan 78 batch 2: six files')
