/// `redox_check`'s parameters.
pub(crate) mod request;
/// `redox_check`'s result.
pub(crate) mod response;

use crate::check::request::CheckRequest;
use crate::check::response::CheckResponse;
use crate::diagnostic::from_redox::diagnostic_from_redox;

/// `redox_check` — transpile, run rustc, report against the `.sdx`.
///
/// Plan 50 (`docs/44 - 50 - MCP Server.md`) leaf 2. The whole body is
/// [`redox_driver::check`] plus a projection, which is the row's shape: this
/// crate contributes no analysis.
///
/// # Why this runs rustc when the language server does not
///
/// Plan 40 §6 put the rustc pass out of the **language server's** scope, and
/// that was right for a server woken on every keystroke: cancellation and
/// debounce are a larger design than the rest of that row. MCP is
/// request/response — one explicit call with no successor to race — so there is
/// nothing to abandon and the thing plan 40 declined is the thing this tool
/// gets cheaply (plan 50 §1.4).
///
/// It does mean a call takes as long as rustc does. §6 discloses that rather
/// than designing around it.
///
/// # Errors
///
/// Only the environment: no rustc, no scratch directory, rustc failing with no
/// positioned diagnostic. A program that does not compile is a **successful**
/// call with `ok: false`, which is what lets a reader tell *your code is wrong*
/// from *this machine cannot answer*.
pub fn check(request: &CheckRequest) -> Result<CheckResponse, String> {
  let path = std::path::Path::new(&request.path);
  let source =
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", request.path))?;

  let outcome = redox_driver::check(path, &source)?;
  Ok(CheckResponse {
    ok: !outcome.failed,
    diagnostics: outcome
      .diagnostics
      .iter()
      .map(diagnostic_from_redox)
      .collect(),
  })
}
