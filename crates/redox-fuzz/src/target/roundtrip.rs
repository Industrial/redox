/// Parse, emit, and hand back the generated Rust for the oracle to check.
///
/// Plan 42 §1.4's invariant, and the row's centrepiece:
///
/// > If `parse_source` accepts the input, `emit`'s output **parses as Rust**.
///
/// `None` when the input did not parse as Redox, which is not a failure — most
/// of a fuzzer's inputs will not.
///
/// # Why this returns instead of asserting
///
/// Under libFuzzer a target runs the code and lets the fuzzer notice a crash;
/// the oracle is what the gate adds on top. Returning the output keeps the
/// *check* in `tests/`, where `syn` is a dev-dependency — so this crate's
/// dependencies stay the three compiler crates and nothing else, which is the
/// Emerald convention doc 04 names.
pub fn roundtrip(data: &[u8]) -> Option<String> {
  let text = String::from_utf8_lossy(data);
  let program = redox_parser::parse_source(&text).ok()?;
  Some(redox_emit::emit(&program))
}
