/// A deterministic function name for a test's description — doc 03 §15.
///
/// ```text
/// "adds two numbers"         adds_two_numbers
/// "rejects a negative input" rejects_a_negative_input
/// "2 + 2 = 4"                _2_2_4
/// ```
///
/// §15 asks for "a deterministic slug of the description", and the rule is:
/// letters and digits lowercased, everything else a single `_`, with leading
/// and trailing separators trimmed. Deterministic matters more than pretty —
/// the golden corpus compares exact bytes.
///
/// A slug beginning with a digit is prefixed with `_`, because a Rust
/// identifier cannot start with one and a description beginning with a number
/// is entirely ordinary. An empty slug becomes `_`, which is a legal name and
/// keeps the emitter total rather than panicking on `test ""`.
pub(crate) fn test_slug(description: &str) -> String {
  let text = description.trim_matches('"');
  let mut out = String::with_capacity(text.len());
  for ch in text.chars() {
    if ch.is_alphanumeric() {
      out.extend(ch.to_lowercase());
    } else if !out.ends_with('_') {
      out.push('_');
    }
  }
  let trimmed = out.trim_matches('_').to_string();
  if trimmed.is_empty() {
    return "_".to_string();
  }
  if trimmed.starts_with(|c: char| c.is_ascii_digit()) {
    return format!("_{trimmed}");
  }
  trimmed
}
