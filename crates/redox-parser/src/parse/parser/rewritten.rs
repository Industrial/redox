/// A captured type's text with the class variable replaced, when the **whole**
/// type is that variable.
///
/// `t` → `Self`, `&t` → `&Self`, `&mut t` → `&mut Self` inside a class; the
/// impl's generic name inside an instance. `Vec<t>` is returned unchanged —
/// see `substitute_var` for why that gap is deliberate.
pub(crate) fn rewritten(text: &str, var: &str, subject: &str) -> String {
  let (prefix, bare) = if let Some(rest) = text.strip_prefix("&mut ") {
    ("&mut ", rest)
  } else if let Some(rest) = text.strip_prefix('&') {
    ("&", rest)
  } else {
    ("", text)
  };

  if bare.trim() != var {
    return text.to_string();
  }
  format!("{prefix}{subject}")
}
