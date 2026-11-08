/// The one-screen usage message.
///
/// Doc 01 §27: `build`, `check`, `fmt`, `watch`, `--version`, `--help` and
/// nothing more. Cargo owns build, test, docs and dependencies. `check`, `fmt`
/// and `watch` are plans 11 and 12, so plan 01 ships only `build`.
pub(crate) const USAGE: &str = "\
redox — a MoonScript-inspired surface for Rust

usage:
  redox build <file.sdx> [-o <output.xrs>]
  redox --version
  redox --help
";
