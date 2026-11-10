//! Every tool reachable over a real MCP session.
//!
//! Plan 50 §5 criterion 6. An **in-process duplex** rather than a spawned
//! process: plan 40 decision 6's choice for plan 40's reason — the same
//! protocol path with no process to manage, and a mock would only test the
//! mock. `tokio::io::duplex` gives the client and the server each end of a
//! pipe, and `rmcp`'s own client drives the handshake, so the frames on the
//! wire are the frames a real client sends.
//!
//! What this file does **not** do is re-assert the answers; `tools.rs` owns
//! those. Here the questions are *is the tool advertised* and *does a call
//! come back as structured content* — and keeping them apart means a failure
//! names which of the two broke.

mod support;

use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use rmcp::transport::async_rw::AsyncRwTransport;
use support::Case;

/// One `tools/call` request.
///
/// A helper rather than four struct expressions, because
/// `CallToolRequestParams` is `#[non_exhaustive]` and a struct expression is
/// rejected with E0639 — the **second** rmcp type in this row to do that, after
/// `ServerConfig` (plan 50 §1.1). Once is a quirk; twice is the crate's
/// convention, and a helper is where a convention belongs.
fn call(name: &'static str, arguments: serde_json::Value) -> CallToolRequestParams {
  let mut params = CallToolRequestParams::default();
  params.name = name.into();
  params.arguments = arguments.as_object().cloned();
  params
}

/// Start a server on one end of an in-process duplex and a client on the other.
async fn connected() -> rmcp::service::RunningService<rmcp::service::RoleClient, ()> {
  let (client_side, server_side) = tokio::io::duplex(1 << 16);
  let (server_read, server_write) = tokio::io::split(server_side);
  tokio::spawn(async move {
    let service = redox_mcp::Server::new()
      .serve(AsyncRwTransport::new(server_read, server_write))
      .await
      .expect("the server should start");
    service.waiting().await.ok();
  });
  let (client_read, client_write) = tokio::io::split(client_side);
  ().serve(AsyncRwTransport::new(client_read, client_write))
    .await
    .expect("the client should complete the handshake")
}

/// All four tools are advertised, with schemas.
#[tokio::test]
async fn the_server_advertises_its_four_tools() {
  let client = connected().await;
  let tools = client
    .list_all_tools()
    .await
    .expect("tools/list should answer");

  let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
  for expected in [
    "redox_check",
    "redox_build",
    "redox_fmt",
    "redox_source_map",
  ] {
    assert!(
      names.contains(&expected),
      "{expected} should be advertised, found {names:?}"
    );
  }
  assert_eq!(names.len(), 4, "and nothing else: {names:?}");

  // The schemas are derived from the request types by `schemars`, so a
  // parameter cannot be documented as something it is not. Asserted because a
  // tool with no input schema is callable and undiscoverable.
  for tool in &tools {
    let schema = serde_json::to_value(&tool.input_schema).expect("should serialise");
    assert_eq!(
      schema["properties"]["path"]["type"], "string",
      "{} should take a string `path`: {schema:#}",
      tool.name
    );
    assert!(
      tool.description.as_ref().is_some_and(|d| d.len() > 20),
      "{} should describe itself — the description is all a client sees",
      tool.name
    );
  }

  client.cancel().await.ok();
}

/// A call over the protocol comes back as **structured content**, not as text.
///
/// The register's requirement, asserted on the wire: *"structured JSON, never
/// scraped stderr"*. A `CallToolResult` carrying only a text block would look
/// like a polite version of scraping, and would pass a test that only checked
/// the call succeeded.
#[tokio::test]
async fn a_call_over_the_protocol_returns_structured_content() {
  if support::skip_without_rustc("a_call_over_the_protocol_returns_structured_content") {
    return;
  }
  let case = Case::new("type_error");
  let client = connected().await;

  let result = client
    .call_tool(call(
      "redox_check",
      serde_json::json!({ "path": case.path() }),
    ))
    .await
    .expect("redox_check should answer");

  let structured = result
    .structured_content
    .as_ref()
    .expect("the result should carry structured content, not only text");
  assert_eq!(
    structured["ok"], false,
    "the fixture does not compile: {structured:#}"
  );
  assert_eq!(
    structured["diagnostics"][0]["code"], "E0308",
    "and rustc's code should survive the round trip: {structured:#}"
  );

  client.cancel().await.ok();
}

/// Every tool is callable over the session — the other three.
///
/// Thin on purpose: the answers are `tools.rs`'s. What this proves is that
/// four `#[tool]` attributes produced four reachable endpoints, which is the
/// thing a macro can silently get wrong.
#[tokio::test]
async fn each_remaining_tool_is_callable() {
  let case = Case::new("clean");
  let client = connected().await;

  for (tool, arguments) in [
    ("redox_build", serde_json::json!({ "path": case.path() })),
    ("redox_fmt", serde_json::json!({ "path": case.path() })),
  ] {
    let result = client
      .call_tool(call(tool, arguments))
      .await
      .unwrap_or_else(|e| panic!("{tool} should answer: {e}"));
    let structured = result
      .structured_content
      .as_ref()
      .unwrap_or_else(|| panic!("{tool} should return structured content"));
    assert_eq!(structured["ok"], true, "{tool}: {structured:#}");
  }

  // `redox_source_map` needs the build above to have written the sidecar,
  // which is why it comes last rather than in the loop.
  let result = client
    .call_tool(call(
      "redox_source_map",
      serde_json::json!({
        "path": case.sibling("clean.xrs").display().to_string(),
        "line": 1,
      }),
    ))
    .await
    .expect("redox_source_map should answer");
  let structured = result
    .structured_content
    .as_ref()
    .expect("structured content");
  assert!(
    structured.get("source").is_some(),
    "it should name the .sdx it is about: {structured:#}"
  );

  client.cancel().await.ok();
}

/// An environment failure is a **protocol error**, not an `ok: false`.
///
/// The other half of `server_content`'s division, over the wire: a reader must
/// be able to tell *your code is wrong* from *this machine cannot answer*, and
/// on the protocol those are different things.
#[tokio::test]
async fn an_unreadable_path_is_a_protocol_error() {
  let client = connected().await;
  let result = client
    .call_tool(call(
      "redox_fmt",
      serde_json::json!({ "path": "/nonexistent/nowhere.sdx" }),
    ))
    .await;
  let error = result.expect_err("a missing file is not an answer about a program");
  assert!(
    error.to_string().contains("cannot read"),
    "and the reason should survive: {error}"
  );
  client.cancel().await.ok();
}
