/// Split an error variant's message into a Rust format string and its
/// positional arguments — doc 03 §12.5.
///
/// ```text
/// "missing key: #{0}"             ("missing key: {}", ["_0"])
/// "invalid #{0} at line #{1}"     ("invalid {} at line {}", ["_0", "_1"])
/// "closed"                        ("closed", [])
/// ```
///
/// The holes are **positional** — `#{0}` names the variant's first field —
/// and they become `{}` with the field's binding passed alongside, rather
/// than Rust's own `{0}`, because the arguments are supplied in hole order
/// and a repeated hole must therefore repeat its argument.
///
/// This is not string interpolation. Plan 23 owns `#{expr}` in a string
/// *expression*; here the holes name a declaration's fields by index, the
/// whole grammar is a decimal integer, and the conversion is local to this
/// construct. Doing it here rather than waiting for plan 23 is what lets
/// §12.5 ship complete.
///
/// A brace the author wrote is doubled, for the reason `double_braces`
/// gives: the result is a format string, so `{` must print as `{`.
pub(crate) fn error_message(raw: &str) -> (String, Vec<String>) {
  let text = raw.trim_matches('"');
  let mut format = String::with_capacity(text.len());
  let mut args = Vec::new();
  let chars: Vec<char> = text.chars().collect();
  let mut i = 0;

  while i < chars.len() {
    if chars[i] == '#' && chars.get(i + 1) == Some(&'{') {
      let mut j = i + 2;
      let mut digits = String::new();
      while j < chars.len() && chars[j].is_ascii_digit() {
        digits.push(chars[j]);
        j += 1;
      }
      if !digits.is_empty() && chars.get(j) == Some(&'}') {
        format.push_str("{}");
        args.push(format!("_{digits}"));
        i = j + 1;
        continue;
      }
    }
    // A brace the author wrote prints as a brace, not as a hole.
    if chars[i] == '{' || chars[i] == '}' {
      format.push(chars[i]);
    }
    format.push(chars[i]);
    i += 1;
  }

  (format, args)
}
