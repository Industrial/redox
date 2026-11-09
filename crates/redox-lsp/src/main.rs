//! The `redox-lsp` binary: stdio transport, then `serve`.
//!
//! Thin on purpose. Plan 40 decision 6 puts the whole server in the library so
//! the tests drive `serve` over an in-process `Connection` — the same protocol
//! path with no process to spawn — and what is left here is the part a test
//! cannot exercise without a terminal.

use lsp_server::Connection;
use lsp_types::InitializeParams;

fn main() -> Result<(), String> {
  let (connection, threads) = Connection::stdio();

  // The handshake has to come before the loop: an editor sends no request
  // until it has the capability list, so a server that started serving first
  // would wait forever on a connection the editor is also waiting on.
  let params = connection
    .initialize(serde_json::to_value(redox_lsp::capabilities()).map_err(|e| e.to_string())?)
    .map_err(|e| e.to_string())?;
  // Deserialised and dropped. Nothing here needs a workspace root — every
  // capability answers from the document the editor sent — and parsing it
  // anyway means a malformed `initialize` fails here rather than somewhere
  // later and stranger.
  let _: InitializeParams = serde_json::from_value(params).map_err(|e| e.to_string())?;

  redox_lsp::serve(&connection)?;
  // The join matters: `lsp-server`'s threads own the stdio handles, and
  // returning without it can drop a reply the editor is still reading.
  threads.join().map_err(|e| e.to_string())
}
