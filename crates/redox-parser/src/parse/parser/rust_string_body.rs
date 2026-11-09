/// Escape decoded text back into the body of a Rust string literal — plan 23.
///
/// The lexer **decodes** escapes: `b"a\nb"` arrives as the three characters
/// `a`, newline, `b`. Re-emitting it needs them escaped again, or the newline
/// lands in the generated source and the literal does not close.
///
/// Two groups are rewritten: the sequences a string body cannot hold — the
/// backslash and the quote — and **every ASCII control character**, as
/// `\xNN`. A control character passed through literally would put an
/// unprintable byte in a human-facing artifact (doc 02 §10.1) and, for `0x7F`
/// in particular, one nothing in a diff or a terminal shows.
///
/// Everything else is passed through, **including every non-ASCII
/// character**: a `.xrs` is UTF-8, and `\u{…}`-escaping readable text would
/// make it neither readable nor shorter.
///
/// The named escapes `\n`, `\r` and `\t` are kept rather than folded into
/// `\xNN`, because those three are what a person reads; the rest of the
/// control range is not.
///
/// Distinct from `redox-emit`'s `rust_escapes`, which takes text that still
/// carries its delimiters and is a *format string*. This one takes a decoded
/// body and produces a plain literal, and conflating the two is how
/// `print_call` once emitted a template with neither transformation applied.
pub(crate) fn rust_string_body(text: &str) -> String {
  let mut out = String::with_capacity(text.len());
  for ch in text.chars() {
    match ch {
      '\\' => out.push_str("\\\\"),
      '"' => out.push_str("\\\""),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      '\0' => out.push_str("\\0"),
      // Every other ASCII control character, `0x7F` included. Found by plan
      // 23's example: `b"\\x7fELF"` decoded to a literal `DEL` byte and was
      // re-emitted as one.
      c if c.is_ascii_control() || c == '\u{7f}' => {
        out.push_str(&format!("\\x{:02X}", c as u32));
      }
      _ => out.push(ch),
    }
  }
  out
}
