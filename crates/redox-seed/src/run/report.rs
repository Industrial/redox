//! One line per file written, on stdout.

use redox_driver::Compiled;

/// Name every output, so a bootstrap log says what the seed produced.
///
/// On **stdout**, where the diagnostics go to stderr: a caller redirecting one
/// and not the other should get the file list without the errors or the errors
/// without the list. `examples/cargo-project/build.rs` records the same split
/// for the opposite reason — a build script's stdout is cargo's directive
/// channel, so its cache reporter prints to stderr instead.
pub(crate) fn report(compiled: &Compiled) {
  for path in &compiled.outputs {
    println!("{}", path.display());
  }
}
