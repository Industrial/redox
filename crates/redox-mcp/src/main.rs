//! The `redox-mcp` binary: stdio transport, then serve until the client goes.
//!
//! Thin on purpose, for plan 40 decision 6's reason one row later: the whole
//! server is in the library so the tests drive it over an in-process duplex —
//! the same protocol path with no process to spawn — and what is left here is
//! the part a test cannot exercise without a pipe.
//!
//! **Stdout belongs to the protocol.** Every diagnostic this binary emits about
//! itself goes to stderr; a stray `println!` here would be injected into the
//! JSON-RPC stream and look to the client like a malformed frame.

use rmcp::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Plan 71 leaf 1, and the same argument as `redox-lsp`'s: stdin belongs to
  // the protocol, so without this the only answer to "what version is this"
  // was `cannot start: connection closed: initialize request`. Checked before
  // the transport is taken, and printed to **stdout** — which is the one
  // `println!` in this binary that is correct, because no protocol stream
  // exists yet.
  if std::env::args()
    .skip(1)
    .any(|arg| arg == "--version" || arg == "-V")
  {
    println!("redox-mcp {}", env!("CARGO_PKG_VERSION"));
    return Ok(());
  }

  let service = redox_mcp::Server::new()
    .serve(rmcp::transport::stdio())
    .await
    .inspect_err(|e| eprintln!("redox-mcp: cannot start: {e}"))?;
  // `waiting` returns when the client closes the transport, which for a stdio
  // server is the normal way to stop. Returning before it would drop replies
  // the client is still reading.
  service.waiting().await?;
  Ok(())
}
