/// The one-screen usage message.
///
/// Doc 01 §27: `build`, `check`, `fmt`, `watch`, `--version`, `--help` and
/// nothing more. Cargo owns build, test, docs and dependencies. Plan 01
/// shipped `build`; plan 07 added `check`. `fmt` is plan 34 and `watch`
/// plan 12.
pub(crate) const USAGE: &str = "\
redox — a MoonScript-inspired surface for Rust

usage:
  redox build <file.sdx> [-o <output.xrs>]
  redox --version
  redox --help
";
