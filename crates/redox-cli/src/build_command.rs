pub(crate) mod build_args;
pub(crate) mod default_output;
pub(crate) mod diagnose;
pub(crate) mod parse_build_args;

use crate::build_command::diagnose::diagnose;
use crate::build_command::parse_build_args::parse_build_args;

/// `redox build <file.sdx> [-o <output.xrs>] [--plain]`.
///
/// Named `build_command` rather than `build` so this module is never confused
/// with the crate's `build.rs` build script, which does an unrelated job.
pub(crate) fn build_command(args: &[String]) -> Result<(), String> {
  let args = parse_build_args(args)?;
  let source = std::fs::read_to_string(&args.input)
    .map_err(|e| format!("error: cannot read {}: {e}", args.input.display()))?;

  let program = redox_parser::parse_source(&source)
    .map_err(|failure| redox_diag::render(&diagnose(&args.input, &source, &failure), args.plain))?;
  let rust = redox_emit::emit(&program);

  std::fs::write(&args.output, &rust)
    .map_err(|e| format!("error: cannot write {}: {e}", args.output.display()))?;
  Ok(())
}
