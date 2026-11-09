pub(crate) mod absorb;
pub(crate) mod answer;
pub(crate) mod cast;
pub(crate) mod diagnostics;
pub(crate) mod documents;
pub(crate) mod formatting;
pub(crate) mod positions;
pub(crate) mod publish;
pub(crate) mod reply;
pub(crate) mod semantic_tokens;
pub(crate) mod symbols;

use crate::serve::absorb::absorb;
use crate::serve::answer::answer;
use crate::serve::documents::Documents;
use crate::serve::publish::publish;
use lsp_server::{Connection, Message};

/// Run the server over a connection until the editor says to stop.
///
/// The connection is a parameter rather than being built here, which is plan
/// 40 decision 6: `lsp-server` offers an in-process pair, so the test drives
/// **this function** over the same protocol path the binary uses, with no
/// process to spawn and no pipes to drain. A hanging test is then a deadlock
/// in the code rather than in the harness.
///
/// # The loop answers everything, including about a document that is broken
///
/// Plan 40's criterion 7. An editor sends requests while the user is
/// mid-keystroke, so a document that does not parse is the **normal** state,
/// not an edge case — and a server that panics there is useless in exactly
/// the moment it is needed. Every handler below takes a `&str` and returns
/// something, which is how that criterion is met structurally rather than by
/// remembering to check.
pub fn serve(connection: &Connection) -> Result<(), String> {
  let mut documents = Documents::new();
  for message in &connection.receiver {
    match message {
      Message::Request(request) => {
        if connection
          .handle_shutdown(&request)
          .map_err(|e| e.to_string())?
        {
          return Ok(());
        }
        let response = answer(&request, &documents);
        connection
          .sender
          .send(Message::Response(response))
          .map_err(|e| e.to_string())?;
      }
      Message::Notification(notification) => {
        if let Some(uri) = absorb(&notification, &mut documents) {
          publish(connection, &uri, &documents)?;
        }
      }
      // A response is the editor answering *our* request, and this server
      // makes none. Ignored rather than logged: an editor is free to send
      // one, and a server that complained would be complaining about
      // conformant behaviour.
      Message::Response(_) => {}
    }
  }
  Ok(())
}
