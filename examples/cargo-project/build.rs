//! Compile this crate's `.sdx` sources into `OUT_DIR`.
//!
//! Doc 02 §6.6 option (a), which plan 09 implements: Voltaic generates `.rs`
//! into Cargo's `OUT_DIR`, and `src/main.rs` brings it in with `include!`.
//! Cargo handles the incremental rebuild, so there is no build system to
//! replace.
//!
//! This file is the whole integration. It is meant to be copied.

/// Print `[cache] <file> HIT|MISS` to **stderr**.
///
/// Stderr and not stdout: a build script's stdout is Cargo's directive
/// channel, so a line printed there is interpreted rather than shown. The
/// cache has no opinion about this — it reports through whatever you give it
/// (plan 10 leaf 4) — which is why the choice is made here, visibly.
struct Report;

impl voltaic_driver::CacheReporter for Report {
  fn report(&self, label: &str, hit: bool) {
    eprintln!("[cache] {label} {}", if hit { "HIT" } else { "MISS" });
  }
}

fn main() {
  let out = std::env::var("OUT_DIR").expect("cargo sets OUT_DIR for a build script");

  match voltaic_driver::compile_dir_with("src".as_ref(), out.as_ref(), &Report) {
    Ok(compiled) => {
      // Without these, Cargo reruns this script only when the crate's own
      // files change — so editing a `.sdx` would not rebuild, and you would
      // silently run stale code.
      for directive in compiled.rerun_directives() {
        println!("{directive}");
      }
      // Rerun if a `.sdx` is *added* or removed, which the per-file
      // directives above cannot express.
      println!("cargo::rerun-if-changed=src");
    }
    Err(diagnostics) => {
      // `plain: true` because Cargo's output is not a terminal: colour codes
      // and width-dependent wrapping would arrive as noise in a build log.
      eprint!("{}", voltaic_diag::render(&diagnostics, true));
      // A build script that returns success after failing to generate its
      // output produces a confusing rustc error about a missing file instead
      // of the Voltaic error that actually happened.
      std::process::exit(1);
    }
  }
}
