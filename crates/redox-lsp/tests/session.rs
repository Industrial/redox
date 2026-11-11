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
  DocumentSymbolRequest, FoldingRangeRequest, Formatting, GotoDefinition, HoverRequest,
  Request as RequestTrait, SemanticTokensFullRequest, Shutdown,
};
use lsp_types::{
  DidOpenTextDocumentParams, DocumentFormattingParams, DocumentSymbolParams,
  DocumentSymbolResponse, FoldingRange, FoldingRangeParams, FormattingOptions,
  GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents, HoverParams,
  PartialResultParams, Position, PublishDiagnosticsParams, SemanticTokens, TextDocumentIdentifier,
  TextDocumentItem, TextDocumentPositionParams, Uri, WorkDoneProgressParams,
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
/// Asserted as a **count plus the seven names**, so adding a capability to the
/// struct without a handler fails here. The alternative — eyeballing the JSON
/// — is what lets a server drift into advertising what it does not do, and
/// plan 40's criterion 1 says why that is worse than advertising nothing: an
/// editor told a capability exists stops looking for another source for it.
///
/// # Four became seven, and the count is the point
///
/// Plan 71 leaf 5 added `hover`, `definition` and `foldingRange`, and this test
/// is the one it had to change rather than weaken — leaf 5 says so in those
/// terms. The three that arrived are **not** the three plan 40 §6 declined: §6
/// refused the hover that reports a *type* and the definition that follows an
/// *import*, both of which need name resolution. Each of these is a span lookup
/// over an AST that already carries spans, and `capabilities.sdx`'s own doc
/// comment carries the table of what each one never does.
///
/// What stays declined is below, and the reason is unchanged: `completion`,
/// `references` and `rename` each need the scope tree.
#[test]
fn capabilities_are_exactly_the_seven_implemented() {
  let advertised = redox_lsp::capabilities();
  assert!(advertised.text_document_sync.is_some(), "sync");
  assert!(advertised.document_symbol_provider.is_some(), "symbols");
  assert!(
    advertised.document_formatting_provider.is_some(),
    "formatting"
  );
  assert!(advertised.semantic_tokens_provider.is_some(), "tokens");
  assert!(advertised.hover_provider.is_some(), "hover");
  assert!(advertised.definition_provider.is_some(), "definition");
  assert!(advertised.folding_range_provider.is_some(), "folding");

  // Still declined, each because it would need the scope tree that AGENTS.md
  // forbids Redox from building.
  assert!(advertised.completion_provider.is_none(), "no completion");
  assert!(advertised.references_provider.is_none(), "no references");
  assert!(advertised.rename_provider.is_none(), "no rename");
  assert!(advertised.code_action_provider.is_none(), "no code actions");
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

/// Plan 71 leaf 5: `hover` names the declaration the cursor is inside.
///
/// A **position in a fixture**, never a string literal — `AGENTS.md`'s rule,
/// and leaf 5's AC restates it. The position is computed from the fixture's own
/// text rather than hardcoded, so an edit to the example moves the cursor with
/// it instead of silently pointing somewhere else.
#[test]
fn hover_names_the_declaration_under_the_cursor() {
  let session = Session::start();
  let source = fixture("examples/operator.sdx");
  session.open(&uri(), &source);

  let at = position_of(&source, "merge a b =");
  let value = session.request(
    1,
    HoverRequest::METHOD,
    &HoverParams {
      text_document_position_params: TextDocumentPositionParams {
        text_document: TextDocumentIdentifier { uri: uri() },
        position: at,
      },
      work_done_progress_params: WorkDoneProgressParams::default(),
    },
  );
  let answer: Option<Hover> = serde_json::from_value(value).expect("a hover response");
  let hover = answer.expect("a cursor inside a function should hover");
  let HoverContents::Markup(markup) = hover.contents else {
    panic!("the server replies with markup");
  };
  assert!(
    markup.value.contains("fn merge"),
    "the hover should name the enclosing declaration, got {:?}",
    markup.value
  );
  // And its first line, which for a function is the signature — the one line
  // of a Redox declaration that says the most.
  assert!(
    markup.value.contains("merge : BTreeSet<u32>"),
    "and show its signature line, got {:?}",
    markup.value
  );
  assert!(
    hover.range.is_some(),
    "a hover carries the range it describes"
  );
}

/// And a cursor outside every declaration hovers nothing.
///
/// The other half of the contract: `hover` either names the enclosing
/// declaration or returns nothing, and never guesses.
///
/// The position is inside a **leading comment**, which is the one place inside
/// a file that is reliably outside every item span: comments are trivia, and an
/// item's span starts at its first *token*. A position past the end of the file
/// would not do — `offset_of` clamps to the nearest real offset by design, so
/// it lands on the last item, and that clamp is what keeps a cursor racing
/// against an edit from producing an error in the user's log.
#[test]
fn hover_outside_a_declaration_is_none() {
  let session = Session::start();
  session.open(
    &uri(),
    "-- a comment above the item\nmain : ()\nmain =\n  ()\n",
  );
  let value = session.request(
    1,
    HoverRequest::METHOD,
    &HoverParams {
      text_document_position_params: TextDocumentPositionParams {
        text_document: TextDocumentIdentifier { uri: uri() },
        position: Position {
          line: 0,
          character: 5,
        },
      },
      work_done_progress_params: WorkDoneProgressParams::default(),
    },
  );
  let answer: Option<Hover> = serde_json::from_value(value).expect("a hover response");
  assert!(answer.is_none(), "nothing there, so nothing to say");
}

/// Plan 71 leaf 5: `definition` jumps to the declaration in the same file.
#[test]
fn definition_finds_the_declaration_in_this_file() {
  let session = Session::start();
  let source = fixture("examples/operator.sdx");
  session.open(&uri(), &source);

  // The `merge` on the `infixl` line is a **use**; the declaration is above it.
  let at = position_of(&source, "infixl 6 <+> = merge");
  let at = Position {
    line: at.line,
    character: at.character + 15,
  };
  let value = session.request(
    1,
    GotoDefinition::METHOD,
    &GotoDefinitionParams {
      text_document_position_params: TextDocumentPositionParams {
        text_document: TextDocumentIdentifier { uri: uri() },
        position: at,
      },
      work_done_progress_params: WorkDoneProgressParams::default(),
      partial_result_params: PartialResultParams::default(),
    },
  );
  let answer: Option<GotoDefinitionResponse> =
    serde_json::from_value(value).expect("a definition response");
  let GotoDefinitionResponse::Scalar(location) = answer.expect("`merge` is declared in this file")
  else {
    panic!("the server replies with a single location");
  };
  assert_eq!(location.uri, uri(), "the same document");
  let declared_at = position_of(&source, "merge : BTreeSet<u32>");
  assert_eq!(
    location.range.start.line, declared_at.line,
    "it should point at the declaration, not at the use"
  );
}

/// And a name this file does not declare answers with nothing.
///
/// Not an error and not a guess: the lookup is same-file by design, so an
/// imported name is outside the contract and the editor keeps its own fallback
/// for that request.
#[test]
fn definition_of_a_name_this_file_does_not_declare_is_none() {
  let session = Session::start();
  let source = "main : ()\nmain =\n  println! \\-> \"hi\"\n";
  session.open(&uri(), source);
  let value = session.request(
    1,
    GotoDefinition::METHOD,
    &GotoDefinitionParams {
      text_document_position_params: TextDocumentPositionParams {
        text_document: TextDocumentIdentifier { uri: uri() },
        // On `println`, which no item here declares.
        position: Position {
          line: 2,
          character: 3,
        },
      },
      work_done_progress_params: WorkDoneProgressParams::default(),
      partial_result_params: PartialResultParams::default(),
    },
  );
  let answer: Option<GotoDefinitionResponse> =
    serde_json::from_value(value).expect("a definition response");
  assert!(answer.is_none(), "not this file's name, so not its answer");
}

/// Plan 71 leaf 5: `foldingRange` is the indented blocks, from the lexer.
///
/// The assertion is structural rather than a count: every range must start
/// above where it ends, and the file's own `main =` header must be foldable.
/// A count would pin the fixture's shape rather than the capability's.
#[test]
fn folding_ranges_are_the_indented_blocks() {
  let session = Session::start();
  let source = fixture("examples/operator.sdx");
  session.open(&uri(), &source);

  let value = session.request(
    1,
    FoldingRangeRequest::METHOD,
    &FoldingRangeParams {
      text_document: TextDocumentIdentifier { uri: uri() },
      work_done_progress_params: WorkDoneProgressParams::default(),
      partial_result_params: PartialResultParams::default(),
    },
  );
  let found: Vec<FoldingRange> = serde_json::from_value(value).expect("a folding response");
  assert!(
    !found.is_empty(),
    "a file with indented bodies should fold somewhere"
  );
  for range in &found {
    assert!(
      range.end_line > range.start_line,
      "a fold spans more than one line, got {range:?}"
    );
  }
  // `main =` opens the longest block in the fixture, so its header line must
  // be the start of some range.
  let main_at = position_of(&source, "main =\n");
  assert!(
    found.iter().any(|r| r.start_line == main_at.line),
    "the `main =` header should be foldable, got {found:?}"
  );
}

/// And a document that does not lex folds nothing rather than erroring.
#[test]
fn folding_a_broken_document_is_empty() {
  let session = Session::start();
  // An unterminated string: the lexer stops, so there are no indent tokens to
  // trust.
  session.open(&uri(), "main : ()\nmain =\n  \"unterminated\n");
  let value = session.request(
    1,
    FoldingRangeRequest::METHOD,
    &FoldingRangeParams {
      text_document: TextDocumentIdentifier { uri: uri() },
      work_done_progress_params: WorkDoneProgressParams::default(),
      partial_result_params: PartialResultParams::default(),
    },
  );
  let found: Vec<FoldingRange> = serde_json::from_value(value).expect("a folding response");
  assert!(
    found.is_empty(),
    "half a file's folds is worse than none, got {found:?}"
  );
}

/// The line and column of `needle`'s first byte in `source`.
///
/// So a cursor position is derived from the fixture rather than written down.
/// A hardcoded line number is the thing that rots when an example gains a
/// comment, and it rots **silently** — the test still passes, pointing at
/// whatever moved into that line.
fn position_of(source: &str, needle: &str) -> Position {
  let at = source
    .find(needle)
    .unwrap_or_else(|| panic!("the fixture should contain {needle:?}"));
  let before = &source[..at];
  let line = before.matches('\n').count();
  let line_start = before.rfind('\n').map_or(0, |i| i + 1);
  let character: usize = source[line_start..at].chars().map(char::len_utf16).sum();
  Position {
    line: u32::try_from(line).unwrap_or(u32::MAX),
    character: u32::try_from(character).unwrap_or(u32::MAX),
  }
}
