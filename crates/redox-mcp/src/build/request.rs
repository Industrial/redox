/// `redox_build`'s parameters.
///
/// No `output`. `redox build` takes `-o`, and this deliberately does not: the
/// output path is `<stem>.xrs` beside the input, which is the rule plan 07's
/// remap and plan 11's build script already rely on, and a client that could
/// redirect it could put the generated Rust somewhere nothing else will look
/// for its source map. Narrowed on purpose; the written paths come back in the
/// response so nothing has to be guessed.
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct BuildRequest {
  /// Path to the `.sdx` file to transpile.
  pub path: String,
}
