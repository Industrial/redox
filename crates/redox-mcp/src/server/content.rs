use rmcp::ErrorData;
use rmcp::model::CallToolResult;

/// One tool's answer as **structured JSON**.
///
/// The register's requirement in one function: *"Each returns plan 05's
/// diagnostics as structured JSON, never scraped stderr."*
/// `CallToolResult::structured` is the MCP shape for a typed result, as against
/// `::success` with a text block — which is what scraping would look like if it
/// were done politely.
///
/// # The two failure kinds stay apart
///
/// Every tool returns `Result<T, String>` where `Err` is reserved for the
/// **environment** — no rustc, an unreadable file, a corrupt map. Those become
/// protocol errors, because they are not answers about the program. A program
/// that does not compile travels the `Ok` path with `ok: false`, so a reader
/// can tell *your code is wrong* from *this machine cannot answer*.
pub(crate) fn server_content<T: serde::Serialize>(
  outcome: Result<T, String>,
) -> Result<CallToolResult, ErrorData> {
  let value = outcome.map_err(|message| ErrorData::internal_error(message, None))?;
  Ok(CallToolResult::structured(
    serde_json::to_value(&value)
      .map_err(|e| ErrorData::internal_error(format!("cannot serialise the result: {e}"), None))?,
  ))
}
