use crate::serve::diagnostics::diagnostics;
use crate::serve::documents::Documents;
use lsp_server::{Connection, Message, Notification};
use lsp_types::notification::{Notification as NotificationTrait, PublishDiagnostics};
use lsp_types::{PublishDiagnosticsParams, Uri};

/// Send the current diagnostics for one document.
pub(crate) fn publish(
  connection: &Connection,
  uri: &Uri,
  documents: &Documents,
) -> Result<(), String> {
  let source = documents.text(uri).unwrap_or_default();
  let params = PublishDiagnosticsParams {
    uri: uri.clone(),
    diagnostics: diagnostics(uri, source),
    version: None,
  };
  connection
    .sender
    .send(Message::Notification(Notification {
      method: PublishDiagnostics::METHOD.to_string(),
      params: serde_json::to_value(params).map_err(|e| e.to_string())?,
    }))
    .map_err(|e| e.to_string())
}
