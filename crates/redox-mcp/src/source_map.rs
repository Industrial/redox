/// `redox_source_map`'s parameters.
pub(crate) mod request;
/// `redox_source_map`'s result.
pub(crate) mod response;

use crate::source_map::request::SourceMapRequest;
use crate::source_map::response::SourceMapResponse;

/// `redox_source_map` — *which `.sdx` line is this `.xrs` line?*
///
/// Plan 50 leaf 3, and the register's fourth tool. The body is
/// [`redox_diag::SourceMap::resolve`], which plan 06 already made public and
/// which answers in a line; the work here is reading the sidecar and being
/// honest about a miss.
///
/// # The sidecar, not an in-memory map
///
/// `redox check` deliberately keeps the map in memory, because the in-memory
/// one carries each origin's span and the sidecar does not (plan 06 §10.3).
/// This tool is the case the sidecar was written **for**: plan 06 made it an
/// artifact *"for external tools"*, and an MCP client holding an `.xrs` and a
/// stack trace is exactly that reader. It asks about a line, not a span.
///
/// # Why a miss is a distinct answer and not an error
///
/// `resolve` returns `None` for two different situations, and plan 06 says a
/// caller *"must handle by reporting against the generated file instead, never
/// by guessing"*:
///
/// - the line is **synthetic** — emitted by Redox with no `.sdx` behind it;
/// - the line is **past the end** of the map.
///
/// `SourceMapResponse` distinguishes them, because *there is no source line*
/// and *you asked about a line that does not exist* want different reactions
/// from a reader. §5 criterion 5.
pub fn source_map(request: &SourceMapRequest) -> Result<SourceMapResponse, String> {
  let xrs = std::path::Path::new(&request.path);
  // The sidecar's name is the full file name plus `.map`, so `main.xrs` has
  // `main.xrs.map`. Accepting either the `.xrs` or the `.map` itself, because
  // a caller holding a stack trace has the `.xrs` and a caller that just ran
  // `redox_build` has both paths from its response.
  let map_path = if xrs.extension().is_some_and(|e| e == "map") {
    xrs.to_path_buf()
  } else {
    let mut name = xrs
      .file_name()
      .ok_or_else(|| format!("{} names no file", request.path))?
      .to_os_string();
    name.push(".map");
    xrs.with_file_name(name)
  };

  let text = std::fs::read_to_string(&map_path).map_err(|e| {
    format!(
      "cannot read {}: {e}. `redox_build` writes it beside the .xrs",
      map_path.display()
    )
  })?;
  let map = redox_diag::parse_map(&text)
    .map_err(|e| format!("{} is not a source map: {e}", map_path.display()))?;

  let column = request.column.unwrap_or(1);
  match map.resolve(request.line, column) {
    Some((line, column)) => Ok(SourceMapResponse {
      source: Some(map.source_path.clone()),
      line: Some(line),
      column: Some(column),
      reason: None,
    }),
    None => Ok(SourceMapResponse {
      source: Some(map.source_path.clone()),
      line: None,
      column: None,
      reason: Some(
        if request.line == 0 || request.line > map.len() {
          "out-of-range: the .xrs has no such line"
        } else {
          "synthetic: Redox emitted this line and no .sdx line is behind it"
        }
        .to_string(),
      ),
    }),
  }
}
