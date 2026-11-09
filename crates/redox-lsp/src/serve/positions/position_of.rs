use lsp_types::Position;

/// One byte offset as a `(line, utf16 character)` position.
///
/// Clamped to the end of the source rather than panicking: a span past the end
/// means the text changed under a diagnostic computed from an older version,
/// which is a race an editor can produce and not a defect to crash on.
pub(crate) fn position_of(source: &str, offset: usize) -> Position {
  let offset = offset.min(source.len());
  let before = &source[..offset];
  let line = before.matches('\n').count();
  let line_start = before.rfind('\n').map_or(0, |at| at + 1);
  let character = source[line_start..offset]
    .chars()
    .map(char::len_utf16)
    .sum::<usize>();
  Position {
    line: u32::try_from(line).unwrap_or(u32::MAX),
    character: u32::try_from(character).unwrap_or(u32::MAX),
  }
}
