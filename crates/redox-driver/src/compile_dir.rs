pub(crate) mod compile_file;
pub(crate) mod compiled;
pub(crate) mod sdx_in;

use crate::compile_dir::compile_file::compile_file;
use crate::compile_dir::compiled::Compiled;
use crate::compile_dir::sdx_in::sdx_in;
use redox_diag::Diagnostic;
use std::path::Path;

/// Compile every `.sdx` in `src` to a `.rs` in `out`.
///
/// Plan 09 (`docs/14 - 09 - Cargo Integration.md`) leaf 1. This is the entry
/// point a user's `build.rs` calls, and the answer to doc 02 §6.6's open
/// question about what a Redox project looks like:
///
/// ```text
/// // build.rs
/// fn main() {
///   let out = std::env::var("OUT_DIR").expect("cargo sets OUT_DIR");
///   match redox_driver::compile_dir("src".as_ref(), out.as_ref()) {
///     Ok(compiled) => {
///       for line in compiled.rerun_directives() {
///         println!("{line}");
///       }
///     }
///     Err(diagnostics) => {
///       eprint!("{}", redox_diag::render(&diagnostics, true));
///       std::process::exit(1);
///     }
///   }
/// }
/// ```
///
/// **`.rs`, not `.xrs`, and deliberately.** rustc accepts `.rs`; `.xrs` stays
/// the canonical human-facing artifact that `redox build` writes for reading
/// and review (doc 02 §6.6). The map sidecar is written beside the `.rs` so
/// plan 07's remapping works through a cargo build.
///
/// **Every error, not the first.** A crate with three bad files reports all
/// three, because a build script runs once and a user who has to fix one error
/// per `cargo build` is being made to pay for the compiler's convenience.
///
/// Errors are returned as [`Diagnostic`]s rather than rendered text: the caller
/// chooses the renderer, and a build script wants the plain one because
/// Cargo's output is not a terminal.
pub fn compile_dir(src: &Path, out: &Path) -> Result<Compiled, Vec<Diagnostic>> {
  let mut compiled = Compiled::default();
  let mut diagnostics = Vec::new();

  for source in sdx_in(src) {
    match compile_file(&source, out) {
      Ok(written) => {
        compiled.inputs.push(source);
        compiled.outputs.push(written);
      }
      Err(mut errors) => diagnostics.append(&mut errors),
    }
  }

  if diagnostics.is_empty() {
    Ok(compiled)
  } else {
    Err(diagnostics)
  }
}
