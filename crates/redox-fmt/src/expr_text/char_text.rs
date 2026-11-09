/// A character literal.
///
/// The one literal the lexer hands over **cooked** — `StrLit`, `RawLit` and
/// `Num` all keep their source text, so this is the only place the formatter
/// has to spell a value rather than copy one.
///
/// `escape_default` is Rust's own escaping, which is also Redox's: it escapes
/// the quote, the backslash and the control characters, and leaves everything
/// printable alone. It spells a non-ASCII character as `\u{…}`, which changes
/// the *text* of a source that wrote the character directly — but not the
/// character, so the program is the same one and the second pass agrees with
/// the first. Both properties hold; only the author's spelling of an
/// exotic literal is lost, which is recorded here rather than discovered.
pub(crate) fn char_text(value: char) -> String {
  format!("'{}'", value.escape_default())
}
