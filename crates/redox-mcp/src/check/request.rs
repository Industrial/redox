/// `redox_check`'s parameters.
///
/// The doc comments below are not decoration: `schemars` turns them into the
/// tool's advertised JSON schema, which is the only description a client ever
/// sees. Plan 50 derives the schema from the type rather than writing it by
/// hand so a parameter cannot be documented as something it is not.
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct CheckRequest {
  /// Path to the `.sdx` file to check.
  pub path: String,
}
