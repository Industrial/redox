use crate::serve::documents::Documents;
use lsp_server::Notification;
use lsp_types::Uri;
use lsp_types::notification::{
  DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
  Notification as NotificationTrait,
};

/// Fold a notification into the document store, reporting the URI whose
/// diagnostics are now stale.
///
/// `None` means nothing to republish — a close, or a notification this server
/// does not track.
pub(crate) fn absorb(notification: &Notification, documents: &mut Documents) -> Option<Uri> {
  match notification.method.as_str() {
    DidOpenTextDocument::METHOD => {
      let params: lsp_types::DidOpenTextDocumentParams =
        serde_json::from_value(notification.params.clone()).ok()?;
      let uri = params.text_document.uri.clone();
      documents.set(uri.clone(), params.text_document.text);
      Some(uri)
    }
    DidChangeTextDocument::METHOD => {
      let params: lsp_types::DidChangeTextDocumentParams =
        serde_json::from_value(notification.params.clone()).ok()?;
      // Full sync, so the last change carries the whole document. Taking the
      // last rather than the first matters: the protocol allows a batch, and
      // with full sync every entry is a complete replacement, so the earlier
      // ones are superseded.
      let text = params.content_changes.into_iter().last()?.text;
      let uri = params.text_document.uri.clone();
      documents.set(uri.clone(), text);
      Some(uri)
    }
    DidCloseTextDocument::METHOD => {
      let params: lsp_types::DidCloseTextDocumentParams =
        serde_json::from_value(notification.params.clone()).ok()?;
      documents.close(&params.text_document.uri);
      None
    }
    _ => None,
  }
}
