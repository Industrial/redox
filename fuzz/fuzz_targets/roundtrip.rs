#![no_main]

// Plan 42 §1.4's invariant, under libFuzzer. The target runs the code and lets
// the fuzzer notice a crash; the *oracle* — does the generated Rust parse? —
// lives in `crates/redox-fuzz/tests/sweep.rs`, where `syn` is a
// dev-dependency. That keeps this wrapper three lines and the target crate's
// dependencies to the three compiler crates, which is the Emerald convention
// doc 04 names.
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
  let _ = redox_fuzz::roundtrip(data);
});
