# -*- coding: utf-8 -*-
"""Plan 77 batch 3: `server`, the four tools and the handler.

Three things in it are new.

**A `Parameters(request)` pattern in a parameter position has no Voltaic
spelling** — a parameter is a bare name — so the whole value is taken and
`params.0` reads it out. `Parameters` is a tuple struct, so that is the same
projection the pattern performed.

**An async method puts the marker on the first type, and `&self` goes second.**
`voltaic_check : async Parameters<CheckRequest> -> &self -> Result<…>` — plan 67
lets the receiver sit at any position, and `emit::signature` writes it first,
which is what keeps the marker off the receiver's own type.

**`move \\-> …` four times**, for `tokio::task::spawn_blocking`. That spelling
is the reason this row had to fix `lambda_text` first: the formatter printed
`\\move  -> …`, which its own parser rejects.

Each `#[tool(description = …)]` is written on **one line**. The Rust wraps it
with `\\`-continuations, and an attribute is carried verbatim — so a wrapped
one would reach the `.rs` with the continuations intact, which is valid Rust
but not something the formatter has a rule for.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

CHECK_DESC = (
    "Check a .sdx file: transpile it, compile the generated Rust with rustc, "
    "and return rustc's diagnostics reported against the .sdx. Returns "
    "{ok, diagnostics}."
)
BUILD_DESC = (
    "Transpile a .sdx file to .xrs beside it, with its .xrs.map source-map "
    "sidecar. Returns {ok, output, map, diagnostics}."
)
FMT_DESC = (
    "Return the canonical formatting of a .sdx file. Reads only — the file is "
    "never written. Returns {ok, formatted, changed, diagnostics}."
)
MAP_DESC = (
    "Which .sdx line is this .xrs line? Resolves a position in generated Rust "
    "back to the source, from the .xrs.map sidecar. Returns "
    "{source, line, column} or {reason} when the line is synthetic or "
    "out of range."
)
INSTRUCTIONS = (
    "Voltaic transpiles .sdx to .xrs containing ordinary Rust. Diagnostics are "
    "reported against the .sdx you wrote, with 1-based line and column in "
    "characters. rustc is the authority on types and borrows — Voltaic has no "
    "checker of its own."
)


def tool(name, request_type, call, desc):
    return '''
  #[tool(description = "%s")]
  %s : async Parameters<%s> -> &self -> Result<CallToolResult, ErrorData>
  %s params self =
    request = params.0
    -- `<-` is the await. `move \\-> …` is the closure `spawn_blocking` wants,
    -- and the reason this row fixed `lambda_text` before writing this file.
    joined <- tokio::task::spawn_blocking (move \\-> %s (&request))
    outcome = (.map_err (\\e -> ErrorData::internal_error (.to_string e) None) joined)?
    server_content outcome
''' % (desc, name, request_type, name, call)


write('server', '''
instance Server
  #[must_use]
  export new : Self
  new = Server
%s%s%s%s
#[tool_handler]
instance ServerHandler Server
  get_info : &self -> ServerConfig
  get_info self =
    -- `ServerConfig` is `#[non_exhaustive]`, so a struct expression is rejected
    -- with E0639 and the fields go on a `Default`. It is also what replaced
    -- the deprecated `ServerInfo` in rmcp 3 — both found by plan 50 §1.1's
    -- spike rather than from a version number.
    mut config = ServerConfig::default ()
    config.capabilities := .build (.enable_tools (ServerCapabilities::builder ()))
    config.instructions := Some (.to_string "%s")
    config
''' % (
    tool('voltaic_check', 'CheckRequest', 'crate::check::check', CHECK_DESC),
    tool('voltaic_build', 'BuildRequest', 'crate::build::build', BUILD_DESC),
    tool('voltaic_fmt', 'FormatRequest', 'crate::format::format', FMT_DESC),
    tool('voltaic_source_map', 'SourceMapRequest', 'crate::source_map::source_map', MAP_DESC),
    INSTRUCTIONS,
))

print('plan 77 batch 3: server')
