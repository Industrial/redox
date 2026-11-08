//! `redox-emit` — the Rust source emitter.
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`), leaf 5. Turns a
//! [`redox_ast::Program`] into Rust source text, written to a `.xrs` file.
//!
//! Three disciplines, each established here because each is expensive to
//! introduce later:
//!
//! - **Deterministic and byte-stable.** No `HashMap` iteration anywhere;
//!   output is a straight fold over the tree in source order. Doc 01 §3.4
//!   requires it, and it is what makes exact-text golden tests possible.
//! - **No rustfmt in the contract** (`docs/COMPILER.md` §5): shelling out
//!   would make output depend on an external tool's version, break
//!   determinism, and destroy the span map plan 06 needs.
//! - **One output line per input line wherever the construct permits**, which
//!   at this slice's size is everywhere except a block's closing brace. Doc 02
//!   §6.5 item 1's line-preserving discipline, which is what keeps plan 06's
//!   span map cheap.
//!
//! Layout follows `docs/06 - Redox Code Architecture.md`: every per-node
//! emitter sits under `emit/`, headed by the entry point.

mod emit;

pub use emit::emit;
