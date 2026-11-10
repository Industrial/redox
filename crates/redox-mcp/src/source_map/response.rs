/// `redox_source_map`'s result.
///
/// A miss is an **answer**, not an error, and the two kinds of miss are
/// distinguished — plan 06 says a caller *"must handle by reporting against the
/// generated file instead, never by guessing"*, and it can only do that if it
/// knows which case it is in:
///
/// - **synthetic** — Redox emitted the line and no `.sdx` line is behind it.
///   Report against the `.xrs`.
/// - **out-of-range** — the `.xrs` has no such line. The caller's line number
///   is wrong, which is a different bug.
///
/// A single `null` for both would make those indistinguishable, and §5
/// criterion 5 exists because that is the easy mistake here.
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct SourceMapResponse {
  /// The `.sdx` the map is about, as the emitter was told it.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub source: Option<String>,
  /// 1-based `.sdx` line. Absent on a miss.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub line: Option<usize>,
  /// 1-based `.sdx` column. Absent on a miss.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub column: Option<usize>,
  /// Why there is no source position, when there is none. `synthetic: …` or
  /// `out-of-range: …`.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reason: Option<String>,
}
