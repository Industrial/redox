/// What [`crate::check`] found.
pub(crate) mod outcome;
/// A temporary directory that removes itself.
pub(crate) mod scratch_dir;

use crate::check::outcome::CheckOutcome;
use crate::check::scratch_dir::ScratchDir;
use crate::diagnose;
use crate::remap;
use crate::remap::rustc_message::RustcMessage;
use crate::rustc_path::rustc_path;
use std::path::Path;
use std::process::Command;

/// Transpile, compile the generated Rust, and report rustc's diagnostics
/// **against the `.sdx`**.
///
/// Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) leaf 4 wrote this as
/// `redox-cli`'s `check_command`. Plan 50 (`docs/44 - 50 - MCP Server.md`)
/// leaf 1 moved the pass here and left the rendering behind, because
/// `redox_check` over MCP needs the same answer as **data** and a copy would
/// let `redox check` and `redox_check` disagree while both compiled.
///
/// rustc's message, code and severity are unchanged; only the position is
/// remapped (plan 07 §1.1).
///
/// # Why `--emit=metadata`
///
/// Type errors are found during analysis, and skipping codegen is the
/// difference between a check that fits in an edit loop and one that does not.
/// It is also what `cargo check` does, for the same reason.
///
/// # Why the map is kept in memory
///
/// The in-memory [`redox_diag::SourceMap`] carries each origin's span and the
/// `.xrs.map` sidecar deliberately does not (plan 06 §10.3) — and the span is
/// what lets a diagnostic underline a whole statement when the column cannot
/// be placed exactly. The sidecar exists for external tools and for plan 11.
///
/// # Errors
///
/// `Err` is for the **environment**, never for the program: no rustc on the
/// system, a scratch directory that cannot be written, or rustc failing with
/// no positioned diagnostic at all. A program that does not compile is a
/// successful check with diagnostics in it, which is what lets a caller tell
/// *your code is wrong* from *this machine cannot answer*.
pub fn check(path: &Path, source: &str) -> Result<CheckOutcome, String> {
  let sdx_path = path.display().to_string();

  // A syntax error is Redox's own, and never reaches rustc.
  let program = match redox_parser::parse_source(source) {
    Ok(program) => program,
    Err(failure) => {
      return Ok(CheckOutcome {
        diagnostics: diagnose(path, source, &failure),
        failed: true,
      });
    }
  };

  // Doc 03 §12.7's lints — plan 38. Reported and **not** fatal: §12.7 makes
  // `unwrap` *"lintable rather than prohibited"*.
  //
  // After the parse, because a warning about a program nobody can compile is
  // noise, and before rustc runs, so Redox's own warnings are not buried under
  // rustc's.
  let mut diagnostics = crate::lint(path, source, Some(&program));

  let (rust, map) = redox_emit::emit_with_map(&program, source, &sdx_path);

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
  let messages: Vec<RustcMessage> = stderr.lines().filter_map(RustcMessage::parse).collect();
  let failed = messages.iter().any(RustcMessage::is_error);
  diagnostics.extend(
    messages
      .iter()
      .map(|m| remap(m, &map, &sdx_path, source, &xrs_path, &rust)),
  );

  if messages.is_empty() && !out.status.success() {
    // rustc failed for a reason it gave no positioned diagnostic for — a
    // missing linker, say. Passing its own words through is more use than a
    // message of Redox's invention.
    return Err(format!("error: rustc failed:\n{}", stderr.trim_end()));
  }

  Ok(CheckOutcome {
    diagnostics,
    failed,
  })
}
