//! The four tools, called directly.
//!
//! Plan 50 (`docs/44 - 50 - MCP Server.md`) §5 criteria 2–5. These assert the
//! **answers**; `session.rs` asserts they are reachable over the protocol.
//! Split that way because a test that goes through MCP to check a line number
//! is testing two things and will be debugged as one.
//!
//! Every assertion here is against the **JSON**, not a rendered string, which
//! is the register's requirement and not a stylistic choice: *"structured JSON,
//! never scraped stderr"* is a claim a test has to make.

mod support;

use support::Case;

/// Criterion 2: a rustc error arrives as data, with a `.sdx` position.
///
/// The code asserted is **rustc's**. Voltaic has no type checker — AGENTS.md:
/// every check rustc can make, rustc makes — so this is the whole value of the
/// tool, and a fixture carrying an `RDX` code would not exercise it. The
/// fixture's own doc comment records that the first one did.
#[test]
fn check_reports_a_rustc_error_against_the_sdx() {
  if support::skip_without_rustc("check_reports_a_rustc_error_against_the_sdx") {
    return;
  }
  let case = Case::new("type_error");
  let response = voltaic_mcp::check(&voltaic_mcp::CheckRequest { path: case.path() })
    .expect("the environment should answer");

  assert!(!response.ok, "the program does not compile");
  let json = serde_json::to_value(&response).expect("should serialise");
  let diagnostics = json["diagnostics"]
    .as_array()
    .expect("diagnostics should be an array");
  assert!(
    !diagnostics.is_empty(),
    "rustc rejects this program, so something should be reported: {json:#}"
  );

  let first = &diagnostics[0];
  assert_eq!(
    first["code"], "E0308",
    "the code should be rustc's own, unrenumbered: {json:#}"
  );
  assert_eq!(first["severity"], "error");
  assert!(
    first["path"]
      .as_str()
      .is_some_and(|p| p.ends_with("type_error.sdx")),
    "reported against the .sdx the user wrote, not the generated .xrs: {json:#}"
  );
  // The mismatched argument is on the last line of the fixture.
  let source = std::fs::read_to_string(&case.sdx).expect("should read the fixture");
  let expected = source
    .lines()
    .position(|l| l.contains("double \"two\""))
    .expect("the fixture should contain the call")
    + 1;
  assert_eq!(
    first["line"].as_u64(),
    Some(expected as u64),
    "and at the line the call is on: {json:#}"
  );
  assert!(
    first["column"].as_u64().is_some_and(|c| c >= 1),
    "columns are 1-based: {json:#}"
  );
  assert!(
    first.get("source").is_none(),
    "the whole file must not travel on the wire (§1.3): {json:#}"
  );
}

/// Criterion 3: a clean program is a success with an empty list, not an error.
#[test]
fn check_on_a_clean_program_reports_nothing() {
  if support::skip_without_rustc("check_on_a_clean_program_reports_nothing") {
    return;
  }
  let case = Case::new("clean");
  let response = voltaic_mcp::check(&voltaic_mcp::CheckRequest { path: case.path() })
    .expect("the environment should answer");
  assert!(response.ok);
  assert!(
    response.diagnostics.is_empty(),
    "a clean program reports nothing: {:?}",
    response.diagnostics
  );
}

/// An unreadable path is an **environment** error, not a diagnostic.
///
/// The distinction `server_content` rests on: a reader has to be able to tell
/// *your code is wrong* from *this machine cannot answer*.
#[test]
fn a_missing_file_is_an_error_and_not_a_diagnostic() {
  let response = voltaic_mcp::check(&voltaic_mcp::CheckRequest {
    path: "/nonexistent/nowhere.sdx".to_string(),
  });
  let message = response.expect_err("a missing file is not an answer about a program");
  assert!(
    message.contains("cannot read"),
    "and should say so plainly: {message}"
  );
}

/// Criterion 4: `voltaic_build` writes the `.xrs` **and** the map, and names
/// both.
#[test]
fn build_writes_the_xrs_and_its_map_and_names_them() {
  let case = Case::new("clean");
  let response = voltaic_mcp::build(&voltaic_mcp::BuildRequest { path: case.path() })
    .expect("the environment should answer");

  assert!(response.ok, "a clean program builds: {response:?}");
  let output = response.output.as_ref().expect("an output path");
  let map = response.map.as_ref().expect("a map path");
  assert!(output.ends_with("clean.xrs"), "{output}");
  assert!(
    map.ends_with("clean.xrs.map"),
    "the sidecar is the stem plus `.map`, which plan 07 relies on: {map}"
  );
  assert!(case.sibling("clean.xrs").is_file(), "the .xrs should exist");
  assert!(
    case.sibling("clean.xrs.map").is_file(),
    "and so should the map — `voltaic_source_map` cannot answer without it"
  );
  let rust = std::fs::read_to_string(case.sibling("clean.xrs")).expect("should read");
  assert!(
    rust.contains("fn main()"),
    "and it should be the generated Rust: {rust}"
  );
}

/// A program that does not parse writes **nothing** and says why.
#[test]
fn build_writes_nothing_when_the_program_does_not_parse() {
  let case = Case::new("parse_error");
  let response = voltaic_mcp::build(&voltaic_mcp::BuildRequest { path: case.path() })
    .expect("a parse failure is an answer about the program, not an environment error");

  assert!(!response.ok);
  assert!(response.output.is_none(), "nothing was written");
  assert!(
    !case.sibling("parse_error.xrs").exists(),
    "and nothing is on disk"
  );
  let json = serde_json::to_value(&response).expect("should serialise");
  let code = json["diagnostics"][0]["code"]
    .as_str()
    .unwrap_or_else(|| panic!("a diagnostic with a code: {json:#}"));
  assert!(
    code.starts_with("RDX"),
    "a parse error is Voltaic's own and never reaches rustc: {code}"
  );
}

/// Criterion 4: `voltaic_fmt` returns the text and does **not** write.
#[test]
fn format_returns_the_canonical_text_and_writes_nothing() {
  let case = Case::new("unformatted");
  let before = std::fs::read_to_string(&case.sdx).expect("should read");

  let response = voltaic_mcp::format(&voltaic_mcp::FormatRequest { path: case.path() })
    .expect("the environment should answer");

  assert!(response.ok);
  assert!(
    response.changed,
    "the fixture is deliberately not canonical"
  );
  let formatted = response.formatted.as_ref().expect("the canonical text");
  assert_ne!(formatted, &before, "and it should differ from the file");
  assert_eq!(
    std::fs::read_to_string(&case.sdx).expect("should read"),
    before,
    "the file on disk must be untouched — the tool reads only (§10)"
  );

  // Idempotence, through this interface: formatting the canonical text again
  // changes nothing. Plan 34 leaf 5's property, asserted here because an MCP
  // caller will round-trip.
  let canonical = case.sibling("canonical.sdx");
  std::fs::write(&canonical, formatted).expect("should write");
  let again = voltaic_mcp::format(&voltaic_mcp::FormatRequest {
    path: canonical.display().to_string(),
  })
  .expect("should answer");
  assert!(!again.changed, "formatting is idempotent");
}

/// Criterion 5: a file that does not parse comes back with its diagnostic and
/// **no** text.
///
/// Plan 34 criterion 8's property through a second interface — *a formatter
/// that rewrites a file it could not understand is the one way this command
/// could destroy work*. Nothing is written either way here, so what is
/// protected is the answer: a refusal rather than a mangled best effort.
#[test]
fn format_refuses_a_file_it_cannot_parse() {
  let case = Case::new("parse_error");
  let before = std::fs::read_to_string(&case.sdx).expect("should read");

  let response = voltaic_mcp::format(&voltaic_mcp::FormatRequest { path: case.path() })
    .expect("a parse failure is an answer about the program");

  assert!(!response.ok);
  assert!(
    response.formatted.is_none(),
    "no best effort: {:?}",
    response.formatted
  );
  assert!(!response.changed);
  assert!(!response.diagnostics.is_empty(), "and it says why");
  assert_eq!(
    std::fs::read_to_string(&case.sdx).expect("should read"),
    before
  );
}

/// Criterion 6: the source map answers, from the sidecar `voltaic_build` wrote.
#[test]
fn source_map_resolves_a_generated_line_to_its_sdx_line() {
  let case = Case::new("clean");
  let built =
    voltaic_mcp::build(&voltaic_mcp::BuildRequest { path: case.path() }).expect("should build");
  let xrs = built.output.expect("an output path");

  // Ask about the line the generated Rust actually puts `print` on, rather
  // than a line number guessed from the fixture — the emitter owns the layout
  // and a hard-coded number would be testing a coincidence.
  let rust = std::fs::read_to_string(&xrs).expect("should read the .xrs");
  let gen_line = rust
    .lines()
    .position(|l| l.contains("println!"))
    .expect("the generated Rust should print")
    + 1;

  let response = voltaic_mcp::source_map(&voltaic_mcp::SourceMapRequest {
    path: xrs.clone(),
    line: gen_line,
    column: None,
  })
  .expect("the sidecar should be readable");

  let json = serde_json::to_value(&response).expect("should serialise");
  assert!(
    response.reason.is_none(),
    "a real line is not a miss: {json:#}"
  );
  let sdx_line = response.line.unwrap_or_else(|| panic!("a line: {json:#}"));
  let source = std::fs::read_to_string(&case.sdx).expect("should read");
  let expected = source
    .lines()
    .position(|l| l.trim_start().starts_with("println! "))
    .expect("the fixture should print")
    + 1;
  assert_eq!(
    sdx_line, expected,
    "the generated print should map back to the source print: {json:#}"
  );
  assert!(
    response
      .source
      .as_ref()
      .is_some_and(|p| p.ends_with("clean.sdx")),
    "and name the .sdx it is about: {json:#}"
  );
}

/// Criterion 6's other half: the two kinds of miss are **distinguishable**.
///
/// Plan 06 says a caller *"must handle by reporting against the generated file
/// instead, never by guessing"*, and it can only choose how if it knows which
/// case it is in. A single `null` for both would make that impossible.
#[test]
fn source_map_distinguishes_a_synthetic_line_from_an_out_of_range_one() {
  let case = Case::new("clean");
  let built =
    voltaic_mcp::build(&voltaic_mcp::BuildRequest { path: case.path() }).expect("should build");
  let xrs = built.output.expect("an output path");

  // A synthetic line: one the emitter wrote with no `.sdx` behind it — a
  // closing brace, or the blank between two items. Found by asking the map
  // rather than by counting braces.
  let map_text =
    std::fs::read_to_string(built.map.expect("a map path")).expect("should read the map");
  let map = voltaic_diag::parse_map(&map_text).expect("should parse");
  let synthetic = *map
    .synthetic_lines()
    .first()
    .expect("the generated Rust has at least a closing brace");

  let response = voltaic_mcp::source_map(&voltaic_mcp::SourceMapRequest {
    path: xrs.clone(),
    line: synthetic,
    column: None,
  })
  .expect("should answer");
  assert!(response.line.is_none());
  let reason = response.reason.as_deref().unwrap_or_default();
  assert!(
    reason.starts_with("synthetic:"),
    "a synthetic line says so: {reason}"
  );

  let past = voltaic_mcp::source_map(&voltaic_mcp::SourceMapRequest {
    path: xrs,
    line: map.len() + 1_000,
    column: None,
  })
  .expect("should answer");
  assert!(past.line.is_none());
  let reason = past.reason.as_deref().unwrap_or_default();
  assert!(
    reason.starts_with("out-of-range:"),
    "and a line that does not exist says something different: {reason}"
  );
}

/// A missing sidecar is an environment error that says how to get one.
#[test]
fn source_map_without_a_sidecar_says_how_to_make_one() {
  let case = Case::new("clean");
  let message = voltaic_mcp::source_map(&voltaic_mcp::SourceMapRequest {
    path: case.sibling("clean.xrs").display().to_string(),
    line: 1,
    column: None,
  })
  .expect_err("there is no sidecar yet");
  assert!(
    message.contains("voltaic_build"),
    "and should name the tool that writes one: {message}"
  );
}
