pub(crate) mod report;

use crate::build_command::parse_build_args::parse_build_args;
use crate::check_command::report::report;

/// `redox check <file.sdx> [--plain]`.
///
/// Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) leaf 4. Transpiles, hands the
/// generated Rust to real rustc, and reports what rustc says **against the
/// `.sdx`** — rustc's message, code and severity unchanged, only the position
/// remapped (plan 07 §1.1).
///
/// Plan 50 (`docs/44 - 50 - MCP Server.md`) leaf 1 moved the pass itself into
/// [`redox_driver::check`], leaving this as what it should always have been:
/// arguments, a file, and [`report`]. The reason is not tidiness — the pass was
/// `pub(crate)` here, `redox_check` over MCP needs the same answer, and a copy
/// would let the two disagree while both compiled. `redox-bench` had already
/// made that copy once (plan 50 §1.2).
pub(crate) fn check_command(args: &[String]) -> Result<(), String> {
  let args = parse_build_args(args)?;
  let source = std::fs::read_to_string(&args.input)
    .map_err(|e| format!("error: cannot read {}: {e}", args.input.display()))?;

  report(&redox_driver::check(&args.input, &source)?, args.plain)
}
