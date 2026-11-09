use crate::remap::char_offset::char_offset;

/// Byte offset of a position in the **generated** file, clamped into range.
///
/// Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) leaf 3. Clamping is correct
/// here and wrong in [`super::narrow`], and the difference is what the caret
/// would be claiming: this offset indexes the `.xrs`, the file rustc was
/// actually talking about, so a clamped position is at worst imprecise about a
/// line the user did not write. A clamped position in the `.sdx` would be a
/// false statement about a line they did.
pub(crate) fn generated_offset(source: &str, line: usize, col: usize) -> usize {
  char_offset(source, line, col).unwrap_or_else(|| source.len().saturating_sub(1))
}

#[cfg(test)]
mod tests {
  use super::generated_offset;

  const SRC: &str = "fn main() {\n}\n";

  #[test]
  fn resolves_a_real_position() {
    assert_eq!(generated_offset(SRC, 2, 1), 12);
    assert_eq!(&SRC[12..13], "}");
  }

  #[test]
  fn clamps_a_position_past_the_end() {
    assert_eq!(generated_offset(SRC, 99, 99), SRC.len() - 1);
  }

  #[test]
  fn survives_an_empty_source() {
    assert_eq!(generated_offset("", 1, 1), 0);
  }
}
