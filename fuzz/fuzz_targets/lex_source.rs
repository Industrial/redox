#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| redox_fuzz::lex_source(data));
