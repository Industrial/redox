//! `redox-mcp` — the driver over the Model Context Protocol.
//!
//! Plan 50 (`docs/44 - 50 - MCP Server.md`), the register's last row. Doc 02
//! §5.5 puts this crate in the layout and §9.3 gives the argument: an agent
//! working in a Redox codebase should get the compiler's answers as **data**
//! rather than as scraped terminal output.
//!
//! Four tools, each a projection of something the compiler already answers:
//!
//! | Tool | Body | Answers |
//! |---|---|---|
//! | `redox_check` | [`redox_driver::check`] | rustc's diagnostics, against the `.sdx` |
//! | `redox_build` | [`redox_emit::emit_with_map`] | the `.xrs` and its map sidecar |
//! | `redox_fmt` | [`redox_fmt::format`] | the canonical `.sdx`, read-only |
//! | `redox_source_map` | [`redox_diag::SourceMap::resolve`] | which `.sdx` line an `.xrs` line came from |
//!
//! **This crate contributes no analysis.** That is the row's whole shape, and
//! it is why leaf 1 came first: the rustc pass was `pub(crate)` inside
//! `redox-cli`, and a copy here would let `redox check` and `redox_check`
//! disagree while both compiled. `redox-bench` had already made that copy
//! once (plan 50 §1.2).
//!
//! # Stdio only
//!
//! No network listener, per the register. `rmcp`'s HTTP and SSE transports and
//! its `client` are off in `Cargo.toml`, and
//! `tests/manifest.rs` asserts it rather than trusting the comment — a claim
//! about a network listener is the kind that stops being true quietly.
//!
//! # The wire shape is a projection, not a serialization
//!
//! [`Diagnostic`] is this crate's own, because `redox_diag::Diagnostic` carries
//! the **whole source file** and **byte offsets** — right for `miette`, wrong
//! for a client that has the file open and quotes a line (plan 50 §1.3).
//!
//! Layout follows `docs/06 - Redox Code Architecture.md`: one construct per
//! file, one module per tool, public items re-exported from here.

/// `redox_build` — transpile a `.sdx` to `.xrs` and its source-map sidecar.
mod build;
/// `redox_check` — transpile, run rustc, report against the `.sdx`.
mod check;
/// One diagnostic, as an MCP client receives it.
mod diagnostic;
/// `redox_fmt` — the canonical form of a `.sdx`, returned and not written.
mod format;
/// The Redox MCP server: four tools over the driver.
mod server;
/// `redox_source_map` — which `.sdx` line an `.xrs` line came from.
mod source_map;

pub use build::build;
pub use build::request::BuildRequest;
pub use build::response::BuildResponse;
pub use check::check;
pub use check::request::CheckRequest;
pub use check::response::CheckResponse;
pub use diagnostic::Diagnostic;
pub use format::format;
pub use format::request::FormatRequest;
pub use format::response::FormatResponse;
pub use server::Server;
pub use source_map::request::SourceMapRequest;
pub use source_map::response::SourceMapResponse;
pub use source_map::source_map;
