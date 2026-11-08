//! `redox` — the command-line interface.
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`), leaf 6.
//!
//! Layout follows `docs/06 - Redox Code Architecture.md`: this module root
//! holds only module declarations and `fn main`, which has no other legal
//! home.

mod build_command;
mod default_output;
mod line_col;
mod parse_build_args;
mod render;
mod run;
mod usage;

use std::process::ExitCode;

fn main() -> ExitCode {
  let args: Vec<String> = std::env::args().skip(1).collect();
  match run::run(&args) {
    Ok(()) => ExitCode::SUCCESS,
    Err(message) => {
      eprintln!("{message}");
      ExitCode::FAILURE
    }
  }
}
