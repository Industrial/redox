pub(crate) mod build_args;
pub(crate) mod default_output;
pub(crate) mod diagnose;
pub(crate) mod map_path;
pub(crate) mod parse_build_args;

use crate::build_command::diagnose::diagnose;
use crate::build_command::map_path::map_path;
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
  let source_path = args.input.display().to_string();
  let (rust, map) = redox_emit::emit_with_map(&program, &source, &source_path);

  std::fs::write(&args.output, &rust)
    .map_err(|e| format!("error: cannot write {}: {e}", args.output.display()))?;

  // The sidecar sits beside the `.xrs` under the same stem plus `.map`, so
  // `main.xrs` has `main.xrs.map`. Plan 07 finds it by that rule rather than
  // by being told where it is.
  let map_path = map_path(&args.output);
  std::fs::write(&map_path, redox_diag::render_map(&map))
    .map_err(|e| format!("error: cannot write {}: {e}", map_path.display()))?;
  Ok(())
}
