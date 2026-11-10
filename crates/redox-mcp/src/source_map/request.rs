/// `redox_source_map`'s parameters.
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct SourceMapRequest {
  /// Path to the generated `.xrs`, or to its `.xrs.map` sidecar directly.
  /// Either works — a caller holding a stack trace has the `.xrs`, and a
  /// caller that just called `redox_build` has both from its response.
  pub path: String,
  /// 1-based line in the `.xrs`.
  pub line: usize,
  /// 1-based column in the `.xrs`. Defaults to 1.
  ///
  /// Optional because the question the register poses — *which `.sdx` line is
  /// this `.xrs` line* — is about a line, and requiring a column would make
  /// every caller invent one.
  #[serde(default)]
  pub column: Option<usize>,
}
