//! `redox-lsp` — the language server.
//!
//! Plan 40 (`docs/35 - 40 - LSP.md`). Four capabilities, each one wired to
//! something that already exists:
//!
//! | Capability | What it calls |
//! |---|---|
//! | Diagnostics on change | `redox_driver::diagnose` and `redox_driver::lint` |
//! | Semantic tokens | `redox_lexer::lex` |
//! | Document symbols | `redox_ast::Program`'s items |
//! | Formatting | `redox_fmt::format` |
//!
//! **The server contributes no analysis of its own**, which is the shape of the
//! row: there is no symbol table to build, because document symbols are the
//! item list, and no diagnostic engine to write, because `Diagnostic` already
//! carries a code, a severity, a span and a message.
//!
//! Completion, hover and go-to-definition are **declined** rather than
//! deferred. All three need name resolution, and AGENTS.md's rule that every
//! check rustc can make rustc makes means Redox has no symbol table to answer
//! them from — so each would be a guess. §6 of the plan names them for the
//! same reason this doc does: a capability list that quietly omits them reads
//! as an oversight.
//!
//! The crate is a library with a thin binary, so the whole server is testable
//! over an in-process `Connection` — the same protocol path with no process
//! management, which is plan 40 decision 6.

mod capabilities;
mod serve;

pub use capabilities::capabilities;
pub use serve::serve;
