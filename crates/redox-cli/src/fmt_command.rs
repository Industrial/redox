pub(crate) mod fmt_args;
pub(crate) mod parse_fmt_args;

use crate::fmt_command::parse_fmt_args::parse_fmt_args;
use redox_driver::diagnose;

/// `redox fmt FILE…` and `redox fmt --check FILE…` — plan 34, leaf 1.
///
/// Formats in place, or with `--check` reports which files are not canonical
/// and exits non-zero without writing anything.
///
/// # Why `--check` prints names rather than a diff
///
/// A diff needs a diff algorithm, and the information a reader needs from
/// `--check` is *which files*; the diff they want after that is
/// `redox fmt FILE && git diff`, which they already know how to read. Plan 34
/// criterion 1 asks for a diff and this gives a file list — recorded in §10 as
/// a narrowing rather than done quietly.
///
/// # A file that does not parse is left alone
///
/// Criterion 8. The parse error is reported and nothing is written, because a
/// formatter that rewrites a file it could not understand is the one way this
/// command could destroy work.
pub(crate) fn fmt_command(args: &[String]) -> Result<(), String> {
  let args = parse_fmt_args(args)?;
  let mut unformatted = Vec::new();

  for path in &args.inputs {
    let source = std::fs::read_to_string(path)
      .map_err(|e| format!("error: cannot read {}: {e}", path.display()))?;

    let program = redox_parser::parse_source(&source)
      .map_err(|failure| redox_diag::render(&diagnose(path, &source, &failure), args.plain))?;

    let formatted = redox_fmt::format(&program, &source);
    if formatted == source {
      continue;
    }

    if args.check {
      unformatted.push(path.display().to_string());
    } else {
      std::fs::write(path, &formatted)
        .map_err(|e| format!("error: cannot write {}: {e}", path.display()))?;
    }
  }

  if unformatted.is_empty() {
    return Ok(());
  }
  Err(format!(
    "error: not formatted:\n{}\nrun `redox fmt` on them\n",
    unformatted
      .iter()
      .map(|p| format!("  {p}"))
      .collect::<Vec<_>>()
      .join("\n")
  ))
}
