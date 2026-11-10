//! What this crate declares, asserted rather than commented.
//!
//! Plan 50 §5 criteria 7 and 8. Two claims in the register are about things no
//! other test can see — *"stdio transport only; no network listener"* and
//! *"a `.mcp.json` entry is committed"* — and both are the kind that stop being
//! true quietly. A feature flipped on by a future `cargo add --features` would
//! open a port without changing a line of Rust.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn manifest() -> String {
  std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
    .expect("should read this crate's Cargo.toml")
}

/// Criterion 7: stdio only.
///
/// Read off the manifest text rather than through a TOML parser, deliberately:
/// a feature can be enabled in the `[dependencies]` table, in a
/// `[target.…dependencies]` table or in `[features]`, and the question here is
/// *does this string appear anywhere in what this crate asks for*. A parser
/// would have to know all three places; the text does not.
#[test]
fn the_manifest_enables_no_network_transport() {
  let text = manifest();
  // `rmcp`'s own feature names, from `cargo add --dry-run rmcp` (plan 50 §1.1).
  const NETWORK: &[&str] = &[
    "server-side-http",
    "client-side-sse",
    "transport-streamable-http-server",
    "transport-streamable-http-server-session",
    "transport-streamable-http-client",
    "transport-streamable-http-client-reqwest",
    "transport-streamable-http-client-unix-socket",
    "reqwest",
    "reqwest-native-tls",
    "auth",
  ];
  let enabled: Vec<&str> = NETWORK
    .iter()
    .copied()
    .filter(|feature| text.contains(&format!("\"{feature}\"")))
    .collect();
  assert!(
    enabled.is_empty(),
    "the register says stdio only, with no network listener. These rmcp \
     features are enabled: {enabled:?}"
  );
  assert!(
    text.contains("\"transport-io\""),
    "and `transport-io` is what provides `rmcp::transport::stdio()`; it is \
     not a default feature (plan 50 §1.1)"
  );
}

/// Nothing in the **shipped** crate is a client.
///
/// `rmcp`'s `client` feature is a dev-dependency only, because `session.rs`
/// drives the server with a real client over an in-process duplex. That is a
/// test, and a test is not the binary — so the assertion is about *where* the
/// feature appears, not whether it appears.
#[test]
fn the_client_feature_is_a_dev_dependency_only() {
  let text = manifest();
  let dev = text
    .find("[dev-dependencies]")
    .expect("this crate has dev-dependencies");
  let shipped = &text[..dev];
  assert!(
    !shipped.contains("\"client\""),
    "the shipped crate must not contain an MCP client"
  );
  assert!(
    text[dev..].contains("\"client\""),
    "and the tests should use one — plan 40 decision 6: a mock tests the mock"
  );
}

/// Criterion 8: `.mcp.json` gains `redox` and keeps every other server.
///
/// The count is asserted against a floor rather than an exact number, because
/// this repository's own agent configuration is not this plan's to freeze — but
/// *nothing was lost* is, and that is what the floor plus the named entries
/// check. §1.5 is the reason: a script that round-tripped this file through a
/// JSON serializer once rewrote the whole of it.
#[test]
fn the_committed_mcp_json_has_redox_and_keeps_the_others() {
  let text = std::fs::read_to_string(root().join(".mcp.json")).expect("should read .mcp.json");
  let parsed: serde_json::Value = serde_json::from_str(&text).expect(".mcp.json should be JSON");
  let servers = parsed["mcpServers"]
    .as_object()
    .expect("mcpServers should be an object");

  let redox = servers
    .get("redox")
    .expect("`redox` should be committed so this repository's own agents get it");
  assert_eq!(redox["type"], "stdio", "stdio transport only");
  assert_eq!(redox["command"], "cargo");
  let args: Vec<&str> = redox["args"]
    .as_array()
    .expect("args should be an array")
    .iter()
    .map(|a| a.as_str().unwrap_or_default())
    .collect();
  assert!(
    args.contains(&"redox-mcp"),
    "and it should run this crate: {args:?}"
  );

  // The fifteen that were there before leaf 4, named so a lost one fails here
  // rather than being noticed by whoever next needs it.
  for other in [
    "chrome-devtools",
    "context7",
    "ferroscope",
    "fetch",
    "filesystem",
    "git",
    "github",
    "lean-ctx",
    "mcp-debugger",
    "memory",
    "playwright",
    "postgres",
    "roam-code",
    "sequential-thinking",
    "time",
  ] {
    assert!(
      servers.contains_key(other),
      "`{other}` was in .mcp.json before plan 50 and must still be"
    );
  }
  assert!(
    servers.len() >= 16,
    "fifteen servers plus redox, at least: {}",
    servers.len()
  );
}

/// The crate is a workspace member, so its tests are in the ordinary gate.
///
/// Unlike `fuzz/`, which plan 42 kept outside the workspace because
/// `libfuzzer-sys` does not build on stable. This one does, so there is no
/// reason for it to be exempt — and a crate nobody's gate builds is a crate
/// that stops building.
#[test]
fn the_crate_is_in_the_workspace() {
  let text = std::fs::read_to_string(root().join("Cargo.toml")).expect("should read Cargo.toml");
  assert!(
    text.contains("\"crates/redox-mcp\""),
    "redox-mcp should be a workspace member"
  );
}
