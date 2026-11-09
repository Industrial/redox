/// The name in the first `pub fn NAME(` of a Rust source.
pub(crate) fn first_public_function(text: &str) -> Option<String> {
  for line in text.lines() {
    let Some(rest) = line.trim_start().strip_prefix("pub fn ") else {
      continue;
    };
    let name: String = rest
      .chars()
      .take_while(|c| c.is_alphanumeric() || *c == '_')
      .collect();
    if !name.is_empty() {
      return Some(name);
    }
  }
  None
}
