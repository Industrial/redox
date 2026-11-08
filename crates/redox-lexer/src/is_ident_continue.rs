/// Whether `b` may continue an identifier already begun.
pub(crate) fn is_ident_continue(b: u8) -> bool {
  b.is_ascii_alphanumeric() || b == b'_'
}
