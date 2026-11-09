#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| redox_fuzz::emit_program(data));
