use crate::serve::cast::cast;
use crate::serve::documents::Documents;
use crate::serve::formatting::formatting;
use crate::serve::reply::reply;
use crate::serve::semantic_tokens::semantic_tokens;
use crate::serve::symbols::symbols;
use lsp_server::{Request, Response};
use lsp_types::request::{
  DocumentSymbolRequest, Formatting, Request as RequestTrait, SemanticTokensFullRequest,
};

/// Answer one request.
///
/// An unknown method gets a result of `null` rather than an error. The
/// protocol allows either, and `null` is the kinder one: a capability this
/// server did not advertise should not have been asked for, and an error reply
/// shows up in the user's log as though they had done something wrong.
pub(crate) fn answer(request: &Request, documents: &Documents) -> Response {
  let id = request.id.clone();
  match request.method.as_str() {
    DocumentSymbolRequest::METHOD => reply(id, cast::<DocumentSymbolRequest>(request), |params| {
      symbols(
        documents
          .text(&params.text_document.uri)
          .unwrap_or_default(),
      )
    }),
    Formatting::METHOD => reply(id, cast::<Formatting>(request), |params| {
      formatting(
        documents
          .text(&params.text_document.uri)
          .unwrap_or_default(),
      )
    }),
    SemanticTokensFullRequest::METHOD => {
      reply(id, cast::<SemanticTokensFullRequest>(request), |params| {
        semantic_tokens(
          documents
            .text(&params.text_document.uri)
            .unwrap_or_default(),
        )
      })
    }
    // A method this server did not advertise gets a `null` **result** rather
    // than an error. The protocol allows either, and `null` is the kinder
    // one: an error reply shows up in the user's log as though they had done
    // something wrong, when it was the editor that asked.
    _ => Response {
      id,
      response_result: Ok(serde_json::Value::Null),
    },
  }
}
