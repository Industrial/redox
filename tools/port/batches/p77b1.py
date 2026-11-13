# -*- coding: utf-8 -*-
"""Plan 77 batch 1: six of `voltaic-mcp`'s nine bodies.

Two shapes appear for the first time in any port, and `examples/async.sdx`
answers both — it is the file plan 39 wrote precisely so they would be
demonstrated rather than described.

**A generic function with a bound** is `server_content : Serialize t => …`,
exactly as that example's `block_on : Future f => f -> f::Output`. The trait is
imported and named bare, because a constraint reads one name.

**An `async fn` with no parameters** puts the marker before the **return**
type: `main : async Result<(), Box<dyn Error>>`, as the example's `noop :
async ()` does. And `.await` is `<-` — `x <- e` binds, `<- e` awaits for
effect only.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('server/content', '''
import serde::Serialize

export(crate) server_content : Serialize t => Result<t, String> -> Result<CallToolResult, ErrorData>
server_content outcome =
  value = (.map_err (\\message -> ErrorData::internal_error message None) outcome)?
  encoded = .map_err (\\e -> ErrorData::internal_error (format! \\-> "cannot serialise the result: #{e}") None) (serde_json::to_value (&value))
  Ok (CallToolResult::structured encoded?)
''')

write('check', '''
export check : &CheckRequest -> Result<CheckResponse, String>
check request =
  path = std::path::Path::new (&request.path)
  -- Bound, because a nested string inside `#{…}` ends the interpolation early.
  shown = .clone request.path
  source = (.map_err (\\e -> format! \\-> "cannot read #{shown}: #{e}") (std::fs::read_to_string path))?

  outcome = (voltaic_driver::check path (&source))?
  Ok (CheckResponse {
    ok = !outcome.failed,
    diagnostics = outcome.diagnostics
      |> .iter
      |> .map diagnostic_from_voltaic
      |> .collect,
  })
''')

write('diagnostic/from_voltaic', '''
export(crate) diagnostic_from_voltaic : &voltaic_diag::Diagnostic -> Diagnostic
diagnostic_from_voltaic diagnostic =
  position = diagnostic_line_and_column (&diagnostic.source) diagnostic.span.0
  severity = case diagnostic.severity of
    Severity::Error -> "error"
    Severity::Warning -> "warning"
  Diagnostic {
    code = .text diagnostic.code,
    severity = .to_string severity,
    message = .clone diagnostic.message,
    path = .clone diagnostic.path,
    line = position.0,
    column = position.1,
    help = .clone diagnostic.help,
  }
''')

write('diagnostic/line_and_column', '''
export(crate) diagnostic_line_and_column : &str -> usize -> (usize, usize)
diagnostic_line_and_column source offset =
  -- `at`, not a rebinding of `offset`: `RDX1003` reports a shadow, and plan 54
  -- leaf 3 made that a lint deliberately.
  at = .min (.len source) offset
  -- Closed ranges: a parenthesised half-open range is an operator section, so
  -- `(0..at)` names both ends. Neither fallback can fire — `at` is clamped and
  -- `line_start` comes from `rfind` on the same prefix.
  prefix = .unwrap_or "" (.get (0..at) source)
  -- Count the newlines before the offset; the last one ends the preceding
  -- line, so the line after it is the one the offset is on.
  line_start = .map_or 0 (\\a -> a + .len_utf8 '\\n') (.rfind '\\n' prefix)
  line = .count (.matches '\\n' prefix) + 1
  -- Characters, not bytes. `chars().count()` over the line prefix is the
  -- whole of the difference from a byte column, and the reason this function
  -- exists at all.
  column = .count (.chars (.unwrap_or "" (.get (line_start..at) source))) + 1
  (line, column)

test "the first byte is one one"
  assert_eq! \\-> (diagnostic_line_and_column "abc" 0) \\-> (1, 1)

test "a later line counts from its own start"
  -- `\\#{1}` escaped: `#{` opens an interpolation, and this fixture is *about*
  -- a program that contains one.
  source = "main : ()\\nmain =\\n  println! \\\\-> \\"\\#{1}\\"\\n"
  at = .expect "should be there" (.find "print" source)
  assert_eq! \\-> (diagnostic_line_and_column source at) \\-> (3, 3)

-- | The property the whole function exists for: a multi-byte character is
-- | **one** column, not two or three.
test "a column counts characters and not bytes"
  source = "-- é\\nx"
  at = .expect "should be there" (.find 'x' source)
  assert_eq! \\-> (diagnostic_line_and_column source at) \\-> (2, 1)

  -- And within the line: `é` is two bytes, so a byte column would say 5.
  line = "aéb"
  b_at = .expect "should be there" (.find 'b' line)
  assert_eq! \\-> b_at \\-> 3 \\-> "the byte offset is 3"
  assert_eq! \\-> (diagnostic_line_and_column line b_at) \\-> (1, 3)

test "an offset past the end clamps"
  source = "ab\\n"
  assert_eq! \\-> (diagnostic_line_and_column source 9999) \\-> (2, 1)
''')

write('format', '''
export format : &FormatRequest -> Result<FormatResponse, String>
format request =
  path = std::path::Path::new (&request.path)
  shown = .clone request.path
  source = (.map_err (\\e -> format! \\-> "cannot read #{shown}: #{e}") (std::fs::read_to_string path))?

  program = case voltaic_parser::parse_source (&source) of
    Ok parsed -> parsed
    Err failure ->
      return Ok (FormatResponse {
        ok = false,
        formatted = None,
        changed = false,
        diagnostics = diagnose path (&source) (&failure)
          |> .iter
          |> .map diagnostic_from_voltaic
          |> .collect,
      })

  formatted = voltaic_fmt::format (&program) (&source)
  Ok (FormatResponse {
    ok = true,
    changed = formatted != source,
    formatted = Some formatted,
    diagnostics = Vec::new (),
  })
''')

write('main', '''
-- The `#[tokio::main]` attribute is **not** repeated here: the translated head
-- already carries it, and two of them make the outer macro see the inner
-- expansion — a plain `fn main`, which rustc reports as *"the `async` keyword
-- is missing from the function declaration"*.
main : async Result<(), Box<dyn std::error::Error>>
main =
  -- Plan 71 leaf 1, and the same argument as `voltaic-lsp`'s: stdin belongs to
  -- the protocol, so without this the only answer to "what version is this"
  -- was `cannot start: connection closed: initialize request`. Checked before
  -- the transport is taken, and printed to **stdout** — which is the one
  -- `println!` in this binary that is correct, because no protocol stream
  -- exists yet.
  wants_version = std::env::args ()
    |> .skip 1
    |> .any (\\arg -> arg == "--version" || arg == "-V")
  if wants_version
    version = env! \\-> "CARGO_PKG_VERSION"
    println! \\-> "voltaic-mcp #{version}"
    return Ok ()

  -- `<-` is doc 30's bind, which is how `.await` is written.
  started <- .serve (rmcp::transport::stdio ()) (voltaic_mcp::Server::new ())
  service = (.inspect_err (\\e -> eprintln! \\-> "voltaic-mcp: cannot start: #{e}") started)?
  -- `waiting` returns when the client closes the transport, which for a stdio
  -- server is the normal way to stop. Returning before it would drop replies
  -- the client is still reading.
  closed <- .waiting service
  closed?
  Ok ()
''')

print('plan 77 batch 1: six files')
