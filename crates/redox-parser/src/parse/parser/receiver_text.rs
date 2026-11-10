/// The Rust receiver a parameter's type lowers to, if it names the class
/// variable or `Self` — doc 18 §2.5.
///
/// ```text
/// &t         → &self
/// &mut Self  → &mut self
/// t          → self
/// u32        → not a receiver
/// ```
///
/// Reads the captured type **text**, because that is what a type is in Redox
/// (doc 03 §14.1): the parser keeps the source spelling rather than a parsed
/// type, so the borrow prefix is a string prefix. That is enough here and
/// nowhere near enough in general, which is why this is the only place that
/// looks inside a type at all.
pub(crate) fn receiver_text(ty: &str, names: &[String]) -> Option<String> {
  let (prefix, bare) = if let Some(rest) = ty.strip_prefix("&mut ") {
    ("&mut ", rest)
  } else if let Some(rest) = ty.strip_prefix('&') {
    ("&", rest)
  } else {
    ("", ty)
  };

  // Several spellings name the implementing type — `Self`, the class
  // variable, and an `instance`'s own target — so this is a list rather than
  // one comparison. `instance Shape Square` writes `area : &Square -> f64`,
  // and that `&Square` is as much the receiver as `&t` is.
  let bare = bare.trim();
  if !names.iter().any(|n| n == bare) {
    return None;
  }
  Some(format!("{prefix}self"))
}
