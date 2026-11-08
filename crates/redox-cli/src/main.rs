//! `redox` — the command-line interface.
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`), leaf 6. Exactly one working
//! subcommand:
//!
//! ```text
//! redox build <file.sdx> [-o <out.xrs>]
//! redox --version
//! redox --help
//! ```
//!
//! No `check`, no `watch`, no `fmt` — those are plans 11 and 12, and doc 01
//! §27 says not to add unnecessary commands. Cargo owns build, test, docs and
//! dependencies; the CLI never grows to compete with it.
//!
//! Diagnostics here are plain one-line messages carrying a file, line and
//! column. `miette` rendering is plan 05; the spans it needs are already
//! collected by the lexer and carried by the AST.

use redox_parser::ParseFailure;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
redox — a MoonScript-inspired surface for Rust

usage:
  redox build <file.sdx> [-o <output.xrs>]
  redox --version
  redox --help
";

fn main() -> ExitCode {
  let args: Vec<String> = std::env::args().skip(1).collect();
  match run(&args) {
    Ok(()) => ExitCode::SUCCESS,
    Err(message) => {
      eprintln!("{message}");
      ExitCode::FAILURE
    }
  }
}

fn run(args: &[String]) -> Result<(), String> {
  match args.first().map(String::as_str) {
    None | Some("--help" | "-h" | "help") => {
      print!("{USAGE}");
      Ok(())
    }
    Some("--version" | "-V") => {
      println!("redox {}", env!("CARGO_PKG_VERSION"));
      Ok(())
    }
    Some("build") => build(&args[1..]),
    Some(other) => Err(format!("error: unknown command `{other}`\n\n{USAGE}")),
  }
}

fn build(args: &[String]) -> Result<(), String> {
  let (input, output) = parse_build_args(args)?;
  let source = std::fs::read_to_string(&input)
    .map_err(|e| format!("error: cannot read {}: {e}", input.display()))?;

  let program = redox_parser::parse_source(&source).map_err(|f| render(&input, &source, &f))?;
  let rust = redox_emit::emit(&program);

  std::fs::write(&output, &rust)
    .map_err(|e| format!("error: cannot write {}: {e}", output.display()))?;
  Ok(())
}

fn parse_build_args(args: &[String]) -> Result<(PathBuf, PathBuf), String> {
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

/// `main.sdx` becomes `main.xrs` (doc 01 §3.4). A path with any other
/// extension, or none, gains `.xrs` rather than losing what it has.
fn default_output(input: &Path) -> PathBuf {
  if input.extension().is_some_and(|e| e == "sdx") {
    input.with_extension("xrs")
  } else {
    let mut name = input.as_os_str().to_os_string();
    name.push(".xrs");
    PathBuf::from(name)
  }
}

/// Render a failure as `error: …` plus a `--> file:line:col` location.
fn render(path: &Path, source: &str, failure: &ParseFailure) -> String {
  let mut out = String::new();
  match failure {
    ParseFailure::Lex(e) => {
      out.push_str(&format!("error: {}\n", e.message));
      out.push_str(&format!("  --> {}:{}:{}\n", path.display(), e.line, e.col));
    }
    ParseFailure::Parse(errors) => {
      for e in errors {
        let (line, col) = line_col(source, e.span.0);
        out.push_str(&format!("error: {}\n", e.message));
        out.push_str(&format!("  --> {}:{line}:{col}\n", path.display()));
      }
    }
  }
  out.pop();
  out
}

/// Resolve a byte offset to a 1-based line and column.
fn line_col(source: &str, offset: usize) -> (usize, usize) {
  let clamped = offset.min(source.len());
  let before = &source[..clamped];
  let line = before.matches('\n').count() + 1;
  let col = before
    .rsplit('\n')
    .next()
    .map_or(1, |l| l.chars().count() + 1);
  (line, col)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn maps_sdx_to_xrs() {
    assert_eq!(
      default_output(Path::new("main.sdx")),
      PathBuf::from("main.xrs")
    );
    assert_eq!(
      default_output(Path::new("a/b/main.sdx")),
      PathBuf::from("a/b/main.xrs")
    );
  }

  #[test]
  fn appends_to_an_unexpected_extension_rather_than_replacing_it() {
    assert_eq!(
      default_output(Path::new("notes.txt")),
      PathBuf::from("notes.txt.xrs")
    );
  }

  #[test]
  fn resolves_a_byte_offset_to_a_line_and_column() {
    let src = "main = ->\n  print \"x\"\n";
    assert_eq!(line_col(src, 0), (1, 1));
    assert_eq!(line_col(src, 12), (2, 3));
  }

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
}
