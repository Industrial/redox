use crate::default_output::default_output;
use crate::usage::USAGE;
use std::path::PathBuf;

/// Resolve `build`'s arguments into an input and an output path.
pub(crate) fn parse_build_args(args: &[String]) -> Result<(PathBuf, PathBuf), String> {
  let mut input: Option<PathBuf> = None;
  let mut output: Option<PathBuf> = None;
  let mut i = 0;
  while i < args.len() {
    match args[i].as_str() {
      "-o" | "--output" => {
        let path = args
          .get(i + 1)
          .ok_or_else(|| "error: -o requires a path".to_string())?;
        output = Some(PathBuf::from(path));
        i += 2;
      }
      flag if flag.starts_with('-') => {
        return Err(format!("error: unknown flag `{flag}`\n\n{USAGE}"));
      }
      path => {
        if input.is_some() {
          return Err(format!("error: unexpected second input `{path}`"));
        }
        input = Some(PathBuf::from(path));
        i += 1;
      }
    }
  }
  let input = input.ok_or_else(|| format!("error: no input file\n\n{USAGE}"))?;
  let output = output.unwrap_or_else(|| default_output(&input));
  Ok((input, output))
}

#[cfg(test)]
mod tests {
  use super::parse_build_args;

  #[test]
  fn rejects_an_unknown_flag() {
    let err = parse_build_args(&["--nope".to_string()]).expect_err("should fail");
    assert!(err.contains("unknown flag"), "{err}");
  }

  #[test]
  fn rejects_a_missing_input() {
    let err = parse_build_args(&[]).expect_err("should fail");
    assert!(err.contains("no input file"), "{err}");
  }

  #[test]
  fn rejects_a_second_input() {
    let args = ["a.sdx".to_string(), "b.sdx".to_string()];
    let err = parse_build_args(&args).expect_err("should fail");
    assert!(err.contains("second input"), "{err}");
  }

  #[test]
  fn rejects_a_dangling_output_flag() {
    let err = parse_build_args(&["-o".to_string()]).expect_err("should fail");
    assert!(err.contains("requires a path"), "{err}");
  }
}
