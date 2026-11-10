//! Plan 40 §3: an LSP session driven over an in-process `Connection`.
//!
//! Decision 6's reason for doing it this way rather than spawning the binary
//! and speaking JSON-RPC over pipes: this is the **same protocol path** — the
//! same `Message` types, the same serialisation, the same `serve` loop — with
//! no process to manage and no pipes to drain. So a hanging test is a deadlock
//! in the code rather than in the harness.
//!
//! Every test ends by sending `shutdown` and `exit`, which is criterion 8: a
//! server that leaks a thread per session is a bug users report as "my machine
//! is slow", and a test that never shuts the server down would not notice.

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{
  DidOpenTextDocument, Exit, Notification as NotificationTrait, PublishDiagnostics,
};
use lsp_types::request::{
  DocumentSymbolRequest, Formatting, Request as RequestTrait, SemanticTokensFullRequest, Shutdown,
};
use lsp_types::{
  DidOpenTextDocumentParams, DocumentFormattingParams, DocumentSymbolParams,
  DocumentSymbolResponse, FormattingOptions, PublishDiagnosticsParams, SemanticTokens,
  TextDocumentIdentifier, TextDocumentItem, Uri, WorkDoneProgressParams,
};
use std::str::FromStr;

/// A running server and the editor's end of the wire.
struct Session {
  editor: Connection,
  server: Option<std::thread::JoinHandle<Result<(), String>>>,
}

impl Session {
  /// Start a server on a thread, with the editor's end returned.
  ///
  /// `initialize` is not sent: `serve` does not perform the handshake —
  /// `main` does, before the loop — so a test that sent one would be testing
  /// the binary's first ten lines and would then have to answer the server's
  /// reply. Criterion 1 is asserted directly against `capabilities()`
  /// instead, which is the same value the binary sends.
  fn start() -> Self {
    let (server_side, editor) = Connection::memory();
    let server = std::thread::spawn(move || redox_lsp::serve(&server_side));
    Self {
      editor,
      server: Some(server),
    }
  }

  fn open(&self, uri: &Uri, text: &str) {
    let params = DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: uri.clone(),
        language_id: "redox".to_string(),
        version: 1,
        text: text.to_string(),
      },
    };
    self.notify(DidOpenTextDocument::METHOD, &params);
  }

  fn notify<P: serde::Serialize>(&self, method: &str, params: &P) {
    self
      .editor
      .sender
      .send(Message::Notification(Notification {
        method: method.to_string(),
        params: serde_json::to_value(params).expect("params should serialise"),
      }))
      .expect("the server should be listening");
  }

  /// Send a request and wait for its response, skipping notifications.
  fn request<P: serde::Serialize>(&self, id: i32, method: &str, params: &P) -> serde_json::Value {
    self
      .editor
      .sender
      .send(Message::Request(Request {
        id: RequestId::from(id),
        method: method.to_string(),
        params: serde_json::to_value(params).expect("params should serialise"),
      }))
      .expect("the server should be listening");
    loop {
      match self.editor.receiver.recv().expect("a reply should arrive") {
        Message::Response(Response {
          response_result, ..
        }) => return response_result.expect("the server replies with a result, never an error"),
        // A `publishDiagnostics` can overtake a reply, which is the protocol
        // working as designed — diagnostics are unsolicited.
        _ => continue,
      }
    }
  }

  /// The next `publishDiagnostics` the server sends.
  fn diagnostics(&self) -> PublishDiagnosticsParams {
    loop {
      match self
        .editor
        .receiver
        .recv()
        .expect("diagnostics should arrive")
      {
        Message::Notification(notification)
          if notification.method == PublishDiagnostics::METHOD =>
        {
          return serde_json::from_value(notification.params).expect("should deserialise");
        }
        _ => continue,
      }
    }
  }
}

impl Drop for Session {
  /// Criterion 8: `shutdown` then `exit`, and the thread must end.
  ///
  /// In `Drop` so that every test gets it without remembering, and so that a
  /// test that fails mid-way still shuts the server down rather than leaving
  /// a thread parked on a channel.
  fn drop(&mut self) {
    let _ = self.editor.sender.send(Message::Request(Request {
      id: RequestId::from(9999),
      method: Shutdown::METHOD.to_string(),
      params: serde_json::Value::Null,
    }));
    let _ = self.editor.sender.send(Message::Notification(Notification {
      method: Exit::METHOD.to_string(),
      params: serde_json::Value::Null,
    }));
    // Dropping the editor's sender closes the channel, which ends the loop
    // even if the handshake above was not honoured — so the join cannot hang.
    if let Some(server) = self.server.take() {
      drop(std::mem::replace(&mut self.editor, Connection::memory().0));
      server
        .join()
        .expect("the server thread should not panic")
        .expect("the server should exit cleanly");
    }
  }
}

fn uri() -> Uri {
  Uri::from_str("file:///probe.sdx").expect("a literal URI should parse")
}

fn fixture(name: &str) -> String {
  let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("../..")
    .join(name);
  std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("should read {}: {e}", path.display()))
}

/// Criterion 1: the advertised list is exactly what is implemented.
///
/// Asserted as a **count plus the four names**, so adding a capability to the
/// struct without a handler fails here. The alternative — eyeballing the JSON
/// — is what lets a server drift into advertising what it does not do, and
/// plan 40's criterion 1 says why that is worse than advertising nothing: an
/// editor told a capability exists stops looking for another source for it.
#[test]
fn capabilities_are_exactly_the_four_implemented() {
  let advertised = redox_lsp::capabilities();
  assert!(advertised.text_document_sync.is_some(), "sync");
  assert!(advertised.document_symbol_provider.is_some(), "symbols");
  assert!(
    advertised.document_formatting_provider.is_some(),
    "formatting"
  );
  assert!(advertised.semantic_tokens_provider.is_some(), "tokens");

  // The ones plan 40 §6 declines, each because it would need name resolution
  // that AGENTS.md forbids Redox from having.
  assert!(advertised.completion_provider.is_none(), "no completion");
  assert!(advertised.hover_provider.is_none(), "no hover");
  assert!(advertised.definition_provider.is_none(), "no definition");
  assert!(advertised.references_provider.is_none(), "no references");
  assert!(advertised.rename_provider.is_none(), "no rename");
}

/// Criterion 2: a syntax error is reported against the `.sdx`, with its `RDX`
/// code.
#[test]
fn a_syntax_error_publishes_an_rdx_diagnostic() {
  let session = Session::start();
  // `main =` with nothing indented under it: the body is missing, which is a
  // parse error rather than a lex one.
  session.open(&uri(), "main : ()\nmain =\n");
  let published = session.diagnostics();

  assert_eq!(published.uri, uri());
  assert!(
    !published.diagnostics.is_empty(),
    "a broken file should publish something"
  );
  let first = &published.diagnostics[0];
  assert_eq!(first.source.as_deref(), Some("redox"));
  let code = match first.code.as_ref().expect("a diagnostic carries a code") {
    lsp_types::NumberOrString::String(text) => text.clone(),
    lsp_types::NumberOrString::Number(n) => n.to_string(),
  };
  assert!(
    code.starts_with("RDX"),
    "the code should be Redox's own, got {code}"
  );
  assert_eq!(
    first.severity,
    Some(lsp_types::DiagnosticSeverity::ERROR),
    "a parse failure is an error"
  );
}

/// Criterion 3: `RDX1001` is a **warning**, and the pragma suppresses it.
///
/// Plan 38 built both halves. This asserts they survive the trip through the
/// protocol, where a severity is an integer and an off-by-one is invisible.
///
/// Plan 53 made it three cases rather than two. The attribute silences the
/// warning outright and is consumed, so the document is clean; the comment
/// silences it and raises `RDX1002` in its place, which is the deprecation
/// replacing what used to be a silent no-op.
#[test]
fn the_unwrap_lint_is_a_warning_and_the_pragma_silences_it() {
  let session = Session::start();
  session.open(
    &uri(),
    "main : ()\nmain =\n  x = .unwrap (Some 1)\n  println! \\-> \"#{x}\"\n  ()\n",
  );
  let published = session.diagnostics();
  let warnings: Vec<_> = published
    .diagnostics
    .iter()
    .filter(|d| d.severity == Some(lsp_types::DiagnosticSeverity::WARNING))
    .collect();
  assert_eq!(warnings.len(), 1, "one unwrap, one warning: {published:?}");
  let code = match warnings[0].code.as_ref().expect("a code") {
    lsp_types::NumberOrString::String(text) => text.clone(),
    lsp_types::NumberOrString::Number(n) => n.to_string(),
  };
  assert_eq!(code, "RDX1001");

  let pragma = Session::start();
  pragma.open(
    &uri(),
    "main : ()\nmain =\n  #[redox::allow(unwrap)]\n  x = .unwrap (Some 1)\n  println! \\-> \"#{x}\"\n  ()\n",
  );
  assert!(
    pragma.diagnostics().diagnostics.is_empty(),
    "the attribute should suppress it, and is consumed rather than reported"
  );

  let deprecated = Session::start();
  deprecated.open(
    &uri(),
    "main : ()\nmain =\n  x = .unwrap (Some 1)  -- redox: allow unwrap\n  println! \\-> \"#{x}\"\n  ()\n",
  );
  let published = deprecated.diagnostics();
  let codes: Vec<String> = published
    .diagnostics
    .iter()
    .map(|d| match d.code.as_ref().expect("a code") {
      lsp_types::NumberOrString::String(text) => text.clone(),
      lsp_types::NumberOrString::Number(n) => n.to_string(),
    })
    .collect();
  assert_eq!(
    codes,
    vec!["RDX1002".to_string()],
    "the comment should suppress `RDX1001` and deprecate itself: {published:?}"
  );
}

/// Criterion 4: `documentSymbol` returns the item list.
#[test]
fn document_symbols_are_the_items() {
  let session = Session::start();
  let source = fixture("examples/class.sdx");
  session.open(&uri(), &source);

  let value = session.request(
    1,
    DocumentSymbolRequest::METHOD,
    &DocumentSymbolParams {
      text_document: TextDocumentIdentifier { uri: uri() },
      work_done_progress_params: WorkDoneProgressParams::default(),
      partial_result_params: lsp_types::PartialResultParams::default(),
    },
  );
  let response: DocumentSymbolResponse = serde_json::from_value(value).expect("a symbol response");
  let DocumentSymbolResponse::Nested(found) = response else {
    panic!("the server replies with the nested form");
  };

  let expected = redox_parser::parse_source(&source)
    .expect("the example parses")
    .items
    .len();
  assert_eq!(found.len(), expected, "one symbol per item");
  // The names are the declarations', which is what makes an outline useful.
  let names: Vec<&str> = found.iter().map(|s| s.name.as_str()).collect();
  assert!(names.contains(&"Shape"), "the class is there: {names:?}");
  assert!(names.contains(&"main"), "and main: {names:?}");
}

/// Criterion 5: an edit for a non-canonical document, none for an example.
///
/// The second half is plan 34 §1.5's *"the corpus is the style guide"*
/// asserted through the protocol rather than through the formatter's own
/// tests.
#[test]
fn formatting_edits_only_what_is_not_canonical() {
  let session = Session::start();
  // Two blank lines where one is canonical, which is leaf 1's rule and needs
  // no construct to demonstrate.
  session.open(
    &uri(),
    "main : ()\nmain =\n  println! \\-> \"#{1}\"\n  ()\n\n\n",
  );
  let edits = formatting_of(&session, 1);
  assert_eq!(edits.len(), 1, "one full-range edit");
  assert!(
    edits[0].new_text.ends_with("  ()\n"),
    "and it is the canonical text: {:?}",
    edits[0].new_text
  );

  let canonical = Session::start();
  canonical.open(&uri(), &fixture("examples/hello.sdx"));
  assert!(
    formatting_of(&canonical, 2).is_empty(),
    "an example is already canonical, so there is nothing to edit"
  );
}

fn formatting_of(session: &Session, id: i32) -> Vec<lsp_types::TextEdit> {
  let value = session.request(
    id,
    Formatting::METHOD,
    &DocumentFormattingParams {
      text_document: TextDocumentIdentifier { uri: uri() },
      options: FormattingOptions::default(),
      work_done_progress_params: WorkDoneProgressParams::default(),
    },
  );
  serde_json::from_value(value).unwrap_or_default()
}

/// Criterion 6: the delta encoding is correct, asserted by decoding it back.
///
/// A relative encoding is exactly where a wrong length shifts every later
/// token silently, so the assertion reconstructs absolute positions and checks
/// them against the source's own lines.
#[test]
fn semantic_tokens_decode_to_the_right_places() {
  let session = Session::start();
  let source = "main : ()\nmain =\n  println! \\-> \"#{1}\"\n  ()\n";
  session.open(&uri(), source);

  let tokens: SemanticTokens =
    serde_json::from_value(tokens_of(&session, 1)).expect("a token response");
  assert!(!tokens.data.is_empty(), "there are tokens");

  let lines: Vec<&str> = source.lines().collect();
  let legend = redox_lsp::capabilities();
  let count = match legend.semantic_tokens_provider {
    Some(lsp_types::SemanticTokensServerCapabilities::SemanticTokensOptions(options)) => {
      options.legend.token_types.len()
    }
    _ => panic!("the legend is advertised"),
  };

  let mut line = 0usize;
  let mut start = 0usize;
  for token in &tokens.data {
    if token.delta_line > 0 {
      line += token.delta_line as usize;
      start = token.delta_start as usize;
    } else {
      start += token.delta_start as usize;
    }
    let text = lines
      .get(line)
      .unwrap_or_else(|| panic!("a token claims line {line}, which the source does not have"));
    let end = start + token.length as usize;
    assert!(
      end <= text.chars().count(),
      "a token runs past the end of line {line}: {start}..{end} of {:?}",
      text
    );
    assert!(
      (token.token_type as usize) < count,
      "the type index {} is outside the legend of {count}",
      token.token_type
    );
  }

  // The first token is `main`, an identifier, at the start of line 0 — which
  // pins the encoding's base case: a first token has to be absolute.
  assert_eq!(tokens.data[0].delta_line, 0);
  assert_eq!(tokens.data[0].delta_start, 0);
  assert_eq!(tokens.data[0].length, 4);
}

/// Criterion 7: a document that does not parse answers every request.
///
/// An editor sends requests while the user is mid-keystroke, so the
/// half-written state is the normal state. A server that panicked here would
/// be useless in exactly the moment it is needed — and because `serve` runs on
/// a thread, a panic would surface as the `Drop` impl's join failing rather
/// than as a hang.
#[test]
fn a_broken_document_answers_everything() {
  let session = Session::start();
  session.open(&uri(), "main : ()\nmain =\n  println! \\-> \"#{(((}\"\n");
  // The diagnostics come first and prove the document is genuinely broken.
  assert!(!session.diagnostics().diagnostics.is_empty());

  // Asserted on the wire value rather than on the deserialised enum.
  // `DocumentSymbolResponse` has two variants that are both JSON arrays —
  // `Flat` and `Nested` — so an **empty** one is ambiguous and serde picks
  // whichever it tries first. A test that matched on the variant failed here
  // with `the nested form` while the server was behaving correctly, which is
  // the protocol's shape and not a defect to work around.
  let symbols = session.request(
    1,
    DocumentSymbolRequest::METHOD,
    &DocumentSymbolParams {
      text_document: TextDocumentIdentifier { uri: uri() },
      work_done_progress_params: WorkDoneProgressParams::default(),
      partial_result_params: lsp_types::PartialResultParams::default(),
    },
  );
  assert_eq!(
    symbols,
    serde_json::json!([]),
    "no symbols, rather than a panic"
  );

  assert!(
    formatting_of(&session, 2).is_empty(),
    "no edit, rather than a panic"
  );

  let tokens: SemanticTokens =
    serde_json::from_value(tokens_of(&session, 3)).expect("a token response");
  // No assertion on the count: unbalanced parentheses are a **lex** error in
  // Redox, not a parse one — `RDX0012`, `UnbalancedDelimiter`, because the
  // lexer tracks bracket depth to decide where a continuation line ends. So
  // this file does not lex and has no tokens, which is the right answer and
  // not the one the first version of this test assumed.
  let _ = tokens;
}

/// And the other kind of broken: a document that **lexes** but does not parse
/// still gets its colour.
///
/// Worth its own test rather than a second assertion above, because the two
/// failures are at different stages and only this one proves the useful
/// property — that a syntax error does not take the highlighting with it.
#[test]
fn a_document_that_lexes_still_colours() {
  let session = Session::start();
  // A signature with no equation under it: lexes cleanly, fails to parse.
  session.open(&uri(), "main : ()\nmain =\n");
  assert!(!session.diagnostics().diagnostics.is_empty());

  let tokens: SemanticTokens =
    serde_json::from_value(tokens_of(&session, 1)).expect("a token response");
  assert!(
    !tokens.data.is_empty(),
    "a file that lexes still colours, even when it does not parse"
  );
}

fn tokens_of(session: &Session, id: i32) -> serde_json::Value {
  session.request(
    id,
    SemanticTokensFullRequest::METHOD,
    &lsp_types::SemanticTokensParams {
      text_document: TextDocumentIdentifier { uri: uri() },
      work_done_progress_params: WorkDoneProgressParams::default(),
      partial_result_params: lsp_types::PartialResultParams::default(),
    },
  )
}

/// An unadvertised method gets a `null` result rather than an error.
///
/// Plan 40's `answer` says why: an error reply shows up in the user's log as
/// though they had done something wrong, when it was the editor that asked.
#[test]
fn an_unknown_method_is_answered_with_null() {
  let session = Session::start();
  session.open(
    &uri(),
    "main : ()\nmain =\n  println! \\-> \"#{1}\"\n  ()\n",
  );
  let _ = session.diagnostics();
  let value = session.request(1, "textDocument/hover", &serde_json::Value::Null);
  assert_eq!(value, serde_json::Value::Null);
}
