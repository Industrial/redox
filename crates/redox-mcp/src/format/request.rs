/// `redox_fmt`'s parameters.
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct FormatRequest {
  /// Path to the `.sdx` file to format. The file is read and never written —
  /// the canonical text comes back in the response.
  pub path: String,
}
