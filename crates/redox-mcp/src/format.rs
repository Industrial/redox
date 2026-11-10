pub(crate) mod request;
pub(crate) mod response;

use crate::diagnostic::from_redox::diagnostic_from_redox;
use crate::format::request::FormatRequest;
use crate::format::response::FormatResponse;
use redox_driver::diagnose;

/// `redox_fmt` — the canonical form of a `.sdx`, **returned and not written**.
///
/// Plan 50 leaf 2, and narrower than §5 criterion 4 asked for. That criterion
/// said *"does not write unless asked"*, which implies a `write` parameter;
/// there is none. A formatter tool that returns text cannot destroy work at
/// all, the client already has the file open, and an agent that wants the
/// change written can write it. Plan 50 §10 records the narrowing.
///
/// # A file that does not parse comes back unchanged
///
/// Plan 34 criterion 8's property, through a second interface: *a formatter
/// that rewrites a file it could not understand is the one way this command
/// could destroy work*. Here it is weaker and still worth asserting — nothing
/// is written either way, so what the property protects is the **answer**: a
/// response whose `formatted` is `None` and whose diagnostics say why, rather
/// than a mangled best effort.
pub fn format(request: &FormatRequest) -> Result<FormatResponse, String> {
  let path = std::path::Path::new(&request.path);
  let source =
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", request.path))?;

  let program = match redox_parser::parse_source(&source) {
    Ok(program) => program,
    Err(failure) => {
      return Ok(FormatResponse {
        ok: false,
        formatted: None,
        changed: false,
        diagnostics: diagnose(path, &source, &failure)
          .iter()
          .map(diagnostic_from_redox)
          .collect(),
      });
    }
  };

  let formatted = redox_fmt::format(&program, &source);
  Ok(FormatResponse {
    ok: true,
    changed: formatted != source,
    formatted: Some(formatted),
    diagnostics: Vec::new(),
  })
}
