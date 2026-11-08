pub(crate) mod remap;
pub(crate) mod rustc_message;
pub(crate) mod rustc_path;
pub(crate) mod scratch_dir;

use crate::build_command::parse_build_args::parse_build_args;
use crate::check_command::remap::remap;
use crate::check_command::rustc_message::RustcMessage;
use crate::check_command::rustc_path::rustc_path;
use crate::check_command::scratch_dir::ScratchDir;
use redox_driver::diagnose;
use std::process::Command;

/// `redox check <file.sdx> [--plain]`.
///
/// Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) leaf 4. Transpiles, hands the
/// generated Rust to real rustc, and reports what rustc says **against the
/// `.sdx`** — rustc's message, code and severity unchanged, only the position
/// remapped (plan 07 §1.1).
///
/// `--emit=metadata` rather than a binary: type errors are found during
/// analysis, and skipping codegen is the difference between a check that fits
/// in an edit loop and one that does not. It is also what `cargo check` does,
/// for the same reason.
///
/// The [`redox_diag::SourceMap`] is kept **in memory** rather than read back
/// from the `.xrs.map` sidecar. The in-memory map carries each origin's span
/// and the sidecar deliberately does not (plan 06 §10.3), and the span is what
/// lets a diagnostic underline a whole statement when the column cannot be
/// placed exactly. The sidecar exists for external tools and for plan 11.
pub(crate) fn check_command(args: &[String]) -> Result<(), String> {
  let args = parse_build_args(args)?;
  let sdx_path = args.input.display().to_string();
  let source = std::fs::read_to_string(&args.input)
    .map_err(|e| format!("error: cannot read {sdx_path}: {e}"))?;

  // A syntax error is Redox's own, and never reaches rustc.
  let program = redox_parser::parse_source(&source)
    .map_err(|failure| redox_diag::render(&diagnose(&args.input, &source, &failure), args.plain))?;
  let (rust, map) = redox_emit::emit_with_map(&program, &source, &sdx_path);

  let rustc = rustc_path().ok_or_else(|| {
    "error: no rustc found. `redox check` compiles the generated Rust, so it needs the \
     Rust toolchain on PATH."
      .to_string()
  })?;

  let dir = ScratchDir::new("check")?;
  let xrs = dir.path().join("check.xrs");
  std::fs::write(&xrs, &rust).map_err(|e| format!("error: cannot write {}: {e}", xrs.display()))?;

  let out = Command::new(&rustc)
    .args([
      "--edition",
      "2024",
      "--error-format=short",
      "--emit=metadata",
    ])
    .arg("-o")
    .arg(dir.path().join("check.rmeta"))
    .arg(&xrs)
    .output()
    .map_err(|e| format!("error: cannot run {}: {e}", rustc.display()))?;

  let xrs_path = xrs.display().to_string();
  let stderr = String::from_utf8_lossy(&out.stderr);
  let diagnostics: Vec<_> = stderr
    .lines()
    .filter_map(RustcMessage::parse)
    .map(|m| remap(&m, &map, &sdx_path, &source, &xrs_path, &rust))
    .collect();

  let failed = stderr
    .lines()
    .filter_map(RustcMessage::parse)
    .any(|m| m.is_error());

  if diagnostics.is_empty() {
    // rustc may still have failed for a reason it gave no positioned
    // diagnostic for — a missing linker, say. Passing its own words through is
    // more use than a message of Redox's invention.
    if !out.status.success() {
      return Err(format!("error: rustc failed:\n{}", stderr.trim_end()));
    }
    return Ok(());
  }

  let rendered = redox_diag::render(&diagnostics, args.plain);
  if failed {
    Err(rendered)
  } else {
    // Warnings go to stderr and do not fail the check, which is rustc's own
    // behaviour and not something Redox should second-guess.
    eprintln!("{rendered}");
    Ok(())
  }
}
