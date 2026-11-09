pub(crate) mod alpha_renamed;
pub(crate) mod body_of;
pub(crate) mod mark_target;
pub(crate) mod strip_debug;

use crate::ir::function_ir::alpha_renamed::alpha_renamed;
use crate::ir::function_ir::body_of::body_of;
use crate::ir::function_ir::mark_target::mark_target;
use crate::ir::rustc_path::rustc_path;
use std::path::Path;

/// One function's optimised LLVM IR, as emitted for `source`.
///
/// # The crate name is held constant, and that is the whole trick
///
/// Rust's symbol mangling includes the crate name and a hash of it, so the same
/// function compiled in two crates has two different symbols and two
/// textually different `define` lines. Both sides compile under
/// `--crate-name probe`, which makes the symbols agree.
///
/// Plan 44 decision 5 prefers that to normalising names afterwards: a
/// post-hoc filter over the IR is a filter that can hide a real difference,
/// and holding the name constant is one flag.
///
/// # `-O` and `--crate-type lib`
///
/// Optimised, because the claim is about the produced binary and an
/// unoptimised comparison would measure the emitter's verbosity instead. A
/// library rather than a binary, so a `main` is not required and the pair's
/// functions survive as `pub` symbols rather than being inlined into nothing.
///
/// Returns `None` when rustc is absent or rejects the input, which the caller
/// reports — swallowing a compile failure here would turn "this does not
/// compile" into "the IR matched", and that is the one wrong answer available.
pub fn function_ir(source: &str, name: &str, scratch: &Path) -> Option<String> {
  let rustc = rustc_path()?;
  std::fs::create_dir_all(scratch).ok()?;
  let input = scratch.join(format!("{name}_probe.rs"));
  let output = scratch.join(format!("{name}_probe.ll"));
  std::fs::write(&input, mark_target(source, name)).ok()?;

  let result = std::process::Command::new(&rustc)
    .arg("-O")
    // One codegen unit. With `-O` rustc splits the module across several,
    // writes one `.ll` per unit, and `-o` then names only one of them — so a
    // function can land in a file this never reads. The symptom is an
    // **empty** body, which compares equal to another empty body: a vacuous
    // pass, which the non-empty assertion in the test is there to catch.
    //
    // It also makes the output deterministic, which a comparison needs more
    // than it needs whatever parallelism buys.
    .arg("-Ccodegen-units=1")
    .arg("--emit=llvm-ir")
    .arg("--crate-type=lib")
    .arg("--crate-name")
    .arg("probe")
    .arg("-o")
    .arg(&output)
    .arg(&input)
    .output()
    .ok()?;
  if !result.status.success() {
    return None;
  }

  let text = std::fs::read_to_string(&output).ok()?;
  Some(alpha_renamed(&body_of(&text, name)))
}
