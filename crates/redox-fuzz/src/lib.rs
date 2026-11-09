//! `redox-fuzz` — the fuzz targets' logic.
//!
//! Plan 42 (`docs/37 - 42 - Fuzzing.md`). Four functions, each taking `&[u8]`
//! and returning nothing, each **total**: no input makes one panic.
//!
//! # Why the logic is here and not in `fuzz/`
//!
//! §1.1 measured it: there is no `cargo-fuzz` and no nightly toolchain on this
//! machine, and neither is pinned. So the libFuzzer harnesses cannot run here,
//! and a fuzz target that has never executed is worse than none — it reads as
//! coverage.
//!
//! The split §1.2 describes fixes that. The logic is stable Rust in the
//! workspace, so `moon run :test` exercises it on every commit through the
//! deterministic sweep in `tests/`; `fuzz/fuzz_targets/*.rs` are three-line
//! wrappers that add a fuzzer when someone has one. The row's value arrives
//! now and grows later.
//!
//! # What each target asserts
//!
//! Three of them assert **totality**: lexing, parsing and emitting never
//! panic. That is a product property rather than hygiene — plan 40's language
//! server runs the parser on every keystroke, so a panic on malformed input
//! crashes an editor.
//!
//! The fourth is the row's centrepiece and §1.4's invariant:
//!
//! > If `parse_source` accepts the input, `emit`'s output **parses as Rust**.
//!
//! The oracle for it lives in the sweep rather than here, because a target's
//! job under libFuzzer is to run the code and let the fuzzer notice a crash;
//! the *check* is what the gate adds on top.

mod target;

pub use target::emit_program::emit_program;
pub use target::lex_source::lex_source;
pub use target::parse_program::parse_program;
pub use target::roundtrip::roundtrip;
