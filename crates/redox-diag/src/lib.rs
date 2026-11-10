//! `redox-diag` — diagnostics: stable codes and `miette` rendering.
//!
//! Plan 05 (`docs/10 - 05 - Diagnostics.md`). Doc 02 §6.5 calls diagnostics
//! the largest single gap in doc 01, whose §29 says "pass Rust errors through"
//! without a mechanism. This crate does the first half — Redox's own errors,
//! rendered with a source snippet and a caret. Remapping *rustc's* errors back
//! to `.sdx` is plan 07 and needs plan 06's span map.
//!
//! Emerald's inception §14.4 states the principle this crate exists to honour:
//! *"Diagnostics are a first-class compiler feature, not an afterthought."*
//!
//! **`miette` reaches this crate and `redox-cli`, and no further.** An error
//! carries a span and a message; only the CLI renders (doc 06 §6 rule 2),
//! which keeps the renderer swappable without the stage crates noticing.
//!
//! Layout follows `docs/06 - Redox Code Architecture.md`.

mod diagnostic;
mod render;
mod source;

pub use diagnostic::Diagnostic;
pub use diagnostic::code::Code;
pub use diagnostic::diagnostic_code::DiagnosticCode;
pub use diagnostic::lint::Lint;
pub use diagnostic::severity::Severity;
pub use render::render;
pub use source::map::SourceMap;
pub use source::map::parse_map::parse_map;
pub use source::map::render_map::render_map;
pub use source::map::version::VERSION as MAP_VERSION;
pub use source::origin::Origin;
pub use source::span::Span;
