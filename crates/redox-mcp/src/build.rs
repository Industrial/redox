pub(crate) mod request;
pub(crate) mod response;

use crate::build::request::BuildRequest;
use crate::build::response::BuildResponse;
use crate::diagnostic::from_redox::diagnostic_from_redox;
use redox_driver::diagnose;

/// `redox_build` — transpile a `.sdx` to `.xrs` and its source-map sidecar.
///
/// Plan 50 leaf 2. The same work `redox build` does, which is why the
/// sidecar is written too: plan 07 finds a map by the rule *`main.xrs` has
/// `main.xrs.map`*, and a build that skipped it would leave `redox_source_map`
/// unable to answer about its own output.
///
/// Unlike [`crate::check::check`] this writes, and the register asks it to —
/// `redox_build` is the tool for *produce the Rust*. The paths written are
/// **returned**, so a caller never has to infer them.
pub fn build(request: &BuildRequest) -> Result<BuildResponse, String> {
  let path = std::path::Path::new(&request.path);
  let source =
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", request.path))?;

  // A syntax error is Redox's own and never reaches the emitter. Returned as a
  // diagnostic rather than an `Err`, because a program that does not parse is
  // an answer about the program (plan 50 §5 criterion 2).
  let program = match redox_parser::parse_source(&source) {
    Ok(program) => program,
    Err(failure) => {
      return Ok(BuildResponse {
        ok: false,
        output: None,
        map: None,
        diagnostics: diagnose(path, &source, &failure)
          .iter()
          .map(diagnostic_from_redox)
          .collect(),
      });
    }
  };

  let diagnostics = redox_driver::lint(path, &source, Some(&program));
  let (rust, map) = redox_emit::emit_with_map(&program, &source, &request.path);

  let output = path.with_extension("xrs");
  std::fs::write(&output, &rust).map_err(|e| format!("cannot write {}: {e}", output.display()))?;

  // Same rule as `redox build`: the stem plus `.map`, so `main.xrs` has
  // `main.xrs.map` rather than `main.map`.
  let mut map_name = output.file_name().expect("a name").to_os_string();
  map_name.push(".map");
  let map_path = output.with_file_name(map_name);
  std::fs::write(&map_path, redox_diag::render_map(&map))
    .map_err(|e| format!("cannot write {}: {e}", map_path.display()))?;

  Ok(BuildResponse {
    ok: true,
    output: Some(output.display().to_string()),
    map: Some(map_path.display().to_string()),
    diagnostics: diagnostics.iter().map(diagnostic_from_redox).collect(),
  })
}
