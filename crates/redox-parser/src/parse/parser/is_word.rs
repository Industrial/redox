/// Whether a character can appear inside an identifier.
///
/// Used to walk a captured type's text, which is a token sequence (doc 03
/// §14.1) — so an identifier in it is delimited by characters this rejects,
/// and a word scan is exact without parsing the type.
pub(crate) fn is_word(ch: char) -> bool {
  ch.is_alphanumeric() || ch == '_'
}
