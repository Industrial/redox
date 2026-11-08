/// Whether `b` may begin an identifier.
pub(crate) fn is_ident_start(b: u8) -> bool {
  b.is_ascii_alphabetic() || b == b'_'
}
