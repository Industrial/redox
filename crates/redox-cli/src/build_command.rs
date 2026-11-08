use crate::parse_build_args::parse_build_args;
use crate::render::render;

/// `redox build <file.sdx> [-o <output.xrs>]`.
///
/// Named `build_command` rather than `build` so this module is never confused
/// with the crate's `build.rs` build script, which does an unrelated job.
pub(crate) fn build_command(args: &[String]) -> Result<(), String> {
  let (input, output) = parse_build_args(args)?;
  let source = std::fs::read_to_string(&input)
    .map_err(|e| format!("error: cannot read {}: {e}", input.display()))?;

  let program = redox_parser::parse_source(&source).map_err(|f| render(&input, &source, &f))?;
  let rust = redox_emit::emit(&program);

  std::fs::write(&output, &rust)
    .map_err(|e| format!("error: cannot write {}: {e}", output.display()))?;
  Ok(())
}
