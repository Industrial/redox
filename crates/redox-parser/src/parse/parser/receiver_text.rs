/// The Rust receiver a parameter's type lowers to, if the type **is** the
/// marker `self` — doc 18 §2.5.
///
/// ```text
/// &self      → &self
/// &mut self  → &mut self
/// self       → self
/// u32        → not a receiver
/// Self       → not a receiver; `unit : f64 -> Self` is an associated fn
/// ```
///
/// Plan 67 replaced a list of *type names* with this one spelling. The list
/// was `Self`, the class variable, and an `instance`'s own target, so a
/// reader needed to know which of up to three names happened to mean the
/// implementing type before they could tell which parameter became `self`.
/// Marking it costs one reserved spelling in type position and deletes both
/// the list and the positional rule that went with it — §1.1.
///
/// The borrow form still comes from the *type*, not from a sigil the author
/// writes, which is what keeps `&`, `&mut` and by-value all expressible with
/// one notation. That half of §2.5 is unchanged, and plan 67 §9 decision 9
/// records why.
///
/// Reads the captured type **text**, because that is what a type is in Redox
/// (doc 03 §14.1): the parser keeps the source spelling rather than a parsed
/// type, so the borrow prefix is a string prefix. That is enough here and
/// nowhere near enough in general, which is why this is the only place that
/// looks inside a type at all.
pub(crate) fn receiver_text(ty: &str) -> Option<String> {
  let (prefix, bare) = if let Some(rest) = ty.strip_prefix("&mut ") {
    ("&mut ", rest)
  } else if let Some(rest) = ty.strip_prefix('&') {
    ("&", rest)
  } else {
    ("", ty)
  };

  if bare.trim() != "self" {
    return None;
  }
  Some(format!("{prefix}self"))
}
