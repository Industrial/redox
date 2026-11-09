use lsp_server::{ExtractError, Request, RequestId, Response};

/// Turn a handler's output into a response, or `null` if the parameters did
/// not deserialise.
pub(crate) fn reply<P, T: serde::Serialize>(
  id: RequestId,
  params: Result<P, ExtractError<Request>>,
  handle: impl FnOnce(P) -> T,
) -> Response {
  match params {
    Ok(params) => Response {
      id,
      response_result: Ok(serde_json::to_value(handle(params)).unwrap_or(serde_json::Value::Null)),
    },
    Err(_) => Response {
      id,
      response_result: Ok(serde_json::Value::Null),
    },
  }
}
