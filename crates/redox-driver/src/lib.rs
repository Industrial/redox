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
//! Layout follows `docs/06 - Redox Code Architecture.md`: one construct per
//! file, modules private, public items re-exported from here.

mod cache;
mod compile_dir;
mod diagnose;
mod lint;
mod remap;

pub use cache::fingerprint::Fingerprint;
pub use cache::reporter::CacheReporter;
pub use cache::reporter::silent::Silent;
pub use compile_dir::compile_dir;
pub use compile_dir::compiled::Compiled;
pub use compile_dir::with::compile_dir_with;
pub use diagnose::diagnose;
pub use lint::lint;
pub use remap::remap;
pub use remap::rustc_message::RustcMessage;
