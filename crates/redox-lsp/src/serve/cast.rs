use lsp_server::{ExtractError, Request};
use lsp_types::request::Request as RequestTrait;

/// Deserialise a request's parameters for a known method.
pub(crate) fn cast<R: RequestTrait>(request: &Request) -> Result<R::Params, ExtractError<Request>> {
  request
    .clone()
    .extract::<R::Params>(R::METHOD)
    .map(|(_, params)| params)
}
