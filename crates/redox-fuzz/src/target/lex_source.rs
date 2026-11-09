/// Lex arbitrary bytes.
///
/// Plan 42's first target, and the only one that takes bytes the others cannot:
/// **invalid UTF-8**. `lex` wants a `&str`, so this is where the conversion
/// happens, and it is where a target that assumed valid input would have been
/// hiding a whole class of input from the fuzzer.
///
/// Lossy conversion rather than a bail-out. `from_utf8_lossy` substitutes
/// `U+FFFD` for each bad sequence, which keeps a malformed byte string as an
/// *input* instead of discarding it — and `U+FFFD` is a character the lexer has
/// never seen, which is exactly the kind of thing worth handing it.
pub fn lex_source(data: &[u8]) {
  let text = String::from_utf8_lossy(data);
  // The result is deliberately dropped: the assertion is that this returns
  // rather than panicking. The sweep in `tests/` is what inspects outcomes.
  let _ = redox_lexer::lex(&text);
}
