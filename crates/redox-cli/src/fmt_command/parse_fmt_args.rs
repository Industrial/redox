use crate::fmt_command::fmt_args::FmtArgs;
use crate::run::usage::usage;
use std::path::PathBuf;

/// Resolve `fmt`'s arguments into a [`FmtArgs`] — plan 34, leaf 1.
///
/// Unlike `build`, several inputs are accepted and required: `fmt` with no file
/// is a no-op that looks like success, and plan 34 §1.5's gate is
/// `redox fmt --check examples/*.sdx`.
pub(crate) fn parse_fmt_args(args: &[String]) -> Result<FmtArgs, String> {
  let mut inputs = Vec::new();
  let mut check = false;
  let mut plain = false;

  for arg in args {
    match arg.as_str() {
      "--check" => check = true,
      "--plain" => plain = true,
      flag if flag.starts_with('-') => {
        return Err(format!("error: unknown flag `{flag}`\n\n{}", usage()));
      }
      path => inputs.push(PathBuf::from(path)),
    }
  }

  if inputs.is_empty() {
    return Err(format!("error: fmt needs at least one .sdx\n\n{}", usage()));
  }

  Ok(FmtArgs {
    inputs,
    check,
    plain,
  })
}
