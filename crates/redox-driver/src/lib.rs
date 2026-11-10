//! `redox-driver` — orchestration: compiling a directory of `.sdx`, the
//! `blake3` query cache, and parallel scheduling.
//!
//! Declared empty by plan 01 leaf 1. Filled by **plan 09**
//! (`cargo-integration`), which added [`compile_dir`] — the function a user's
//! `build.rs` calls, and doc 02 §6.6's answer to what a Redox project looks
//! like. **Plan 10** (`incremental-cache`) puts a cache behind the same
//! signature, lifting `emerald-driver/src/cache.rs` (doc 02 §7.1) including
//! the trick of hashing the compiler's own binary into every key.
//!
//! This crate sits above the stages and below the CLI, which is why
//! [`diagnose`] lives here: plan 05 wrote it in the CLI when the CLI was its
//! only caller, and plan 09 gave it a second one that must not depend on the
//! CLI (plan 09 §2.1).
//!
//! **Plan 50** (`mcp-server`) leaf 1 did the same thing for the rustc pass, one
//! row later and for the same reason with the evidence already in: [`check`]
//! and [`rustc_path`] were `pub(crate)` inside `redox-cli`, and the copy that
//! argument predicts had **already been made** — `redox-bench`'s build script
//! was `redox-cli`'s with different doc comments. A third reader
//! (`redox-mcp`'s `redox_check`) is where that stops being tolerable, so this
//! crate now owns the pass, the toolchain resolution, and the `build.rs` that
//! bakes a verified `REDOX_RUSTC` — which must live beside them, because
//! `cargo::rustc-env` bakes into whichever crate carries the script.
//!
//! Layout follows `docs/06 - Redox Code Architecture.md`: one construct per
//! file, modules private, public items re-exported from here.

mod cache;
/// Transpile, compile the generated Rust, and report rustc against the `.sdx`.
mod check;
/// Compile every `.sdx` in a directory to a `.rs` in another.
mod compile_dir;
/// Turn a parse failure into the diagnostics that represent it.
mod diagnose;
/// Every lint, as warnings.
mod lint;
/// Turn one of rustc's diagnostics into one against the `.sdx`.
mod remap;
/// A rustc that behaves like rustc, resolved at run time.
mod rustc_path;

pub use cache::fingerprint::Fingerprint;
pub use cache::reporter::CacheReporter;
pub use cache::reporter::silent::Silent;
pub use check::check;
pub use check::outcome::CheckOutcome;
pub use compile_dir::compile_dir;
pub use compile_dir::compiled::Compiled;
pub use compile_dir::with::compile_dir_with;
pub use diagnose::diagnose;
pub use lint::lint;
pub use remap::remap;
pub use remap::rustc_message::RustcMessage;
pub use rustc_path::rustc_path;
