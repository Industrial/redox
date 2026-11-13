#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| voltaic_fuzz::lex_source(data));
