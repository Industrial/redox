//! Two positional paths, parsed without a dependency.

use std::path::PathBuf;

/// The source directory and the output directory, or `None` for misuse.
///
/// Hand-rolled rather than `clap`, for the reason this crate's module doc
/// gives: a seed with two positional arguments does not need a parser, and
/// every dependency is one more thing that has to compile for a target with no
/// processes and no terminal.
///
/// Anything other than exactly two arguments is misuse, `--help` included.
/// A seed is run by `init.sh` and by CI, not by a person exploring it.
pub(crate) fn arguments() -> Option<(PathBuf, PathBuf)> {
  let mut args = std::env::args_os().skip(1);
  let src = args.next()?;
  let out = args.next()?;
  if args.next().is_some() {
    return None;
  }
  Some((PathBuf::from(src), PathBuf::from(out)))
}

#[cfg(test)]
mod tests {
  use super::arguments;

  /// Inline because `arguments` is private and reads the real process
  /// arguments, which an integration test cannot set without spawning — and
  /// spawning is the one thing the seed's target cannot do. AGENTS.md's rule
  /// is that inline tests are for private constructs, which this is.
  ///
  /// Under `cargo test` the arguments are the harness's, so the only
  /// behaviour assertable here is that it does not panic on whatever it is
  /// handed. The real coverage is `tests/seed.rs`, which runs the binary.
  #[test]
  fn it_does_not_panic_on_the_harness_arguments() {
    let _ = arguments();
  }
}
