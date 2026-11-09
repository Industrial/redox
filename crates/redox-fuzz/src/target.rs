//! The four fuzz targets.
//!
//! A grouping module (doc 06 §1.2): four peers with no natural head, and
//! `target` is the domain word rather than a bucket — these are *the targets*,
//! which is what `fuzz/fuzz_targets/` wraps one-for-one.
//!
//! Three of them assert **totality**: lexing, parsing and emitting never
//! panic, for any bytes at all. The fourth, `roundtrip`, hands back the
//! generated Rust so the sweep can check plan 42 §1.4's invariant — that the
//! emitter never produces Rust which does not parse.

pub(crate) mod emit_program;
pub(crate) mod lex_source;
pub(crate) mod parse_program;
pub(crate) mod roundtrip;
