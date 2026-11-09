/// Parse arbitrary bytes.
///
/// The target that matters most for an editor: plan 40's language server runs
/// `parse_source` on every keystroke, so a panic here is a crash in front of
/// someone who merely has not finished typing. Totality is a product property,
/// not hygiene.
pub fn parse_program(data: &[u8]) {
  let text = String::from_utf8_lossy(data);
  let _ = redox_parser::parse_source(&text);
}
