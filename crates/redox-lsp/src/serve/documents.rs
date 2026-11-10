use lsp_types::Uri;
use std::collections::HashMap;

/// The open documents, keyed by URI.
///
/// Full-document sync (plan 40 decision 5) makes this a map of URI to text and
/// nothing more: every change replaces the whole string, so the store holds
/// exactly what the editor holds and a diagnostic's byte offset is an offset
/// into text the user can see.
///
/// # Why the server reads here and not from disk
///
/// The file on disk is **stale by definition** while the user is typing, and
/// the whole value of a language server is answering about the unsaved buffer.
/// A server that read the path would report yesterday's errors with today's
/// line numbers, which is worse than reporting nothing.
#[derive(Debug, Default)]
pub(crate) struct Documents {
  /// The text of each open document, as the editor currently holds it.
  open: HashMap<Uri, String>,
}

impl Documents {
  /// An empty store, holding no open document.
  pub(crate) fn new() -> Self {
    Self::default()
  }

  /// Record a document's current text, replacing any previous version.
  pub(crate) fn set(&mut self, uri: Uri, text: String) {
    self.open.insert(uri, text);
  }

  /// Forget a closed document.
  pub(crate) fn close(&mut self, uri: &Uri) {
    self.open.remove(uri);
  }

  /// A document's text, or `None` if the editor never opened it.
  ///
  /// `None` is not an error to report. An editor may send a request for a
  /// document it has already closed — the two cross on the wire — so a server
  /// that treated it as a protocol violation would log noise at exactly the
  /// moment the user did nothing wrong.
  pub(crate) fn text(&self, uri: &Uri) -> Option<&str> {
    self.open.get(uri).map(String::as_str)
  }

  // No `len` or `is_empty`. Both were written for "the tests that assert the
  // store is maintained", and no test wants them: `didOpen` followed by a
  // request that answers from the document is already that assertion, and it
  // asserts the behaviour rather than the bookkeeping. An accessor justified
  // by a test that does not exist is dead code with an alibi.
}
