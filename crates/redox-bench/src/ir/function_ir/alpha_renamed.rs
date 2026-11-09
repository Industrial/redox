/// One function body with its local value names renumbered by first
/// appearance.
///
/// # Why this normalisation is safe and the one decision 5 declined is not
///
/// LLVM local names carry the Rust binding's name, so the generated
/// `Ok(__rdx_v) => __rdx_v` and the hand-written `Ok(value) => value` differ by
/// `%__rdx_v.sroa.0.0` against `%value.sroa.0.0` — and by nothing else. The
/// `catch` pair's 240-line body differed in exactly those two lines.
///
/// That is a **binding name**, not a cost. Renaming locals to `%0`, `%1`, …
/// in order of first appearance is an alpha-renaming, and alpha-equivalence is
/// precisely the equivalence doc 03's claim means.
///
/// It is also structure-preserving, which is what separates it from the
/// normalisation plan 44 decision 5 declined. The numbering follows **first
/// use**, so two bodies that use their values in a different order get
/// different numbers and still differ. Decision 5 declined normalising
/// *symbol* names — a hash of the crate and path — because that is a filter
/// over something a real difference could hide behind; the crate name is held
/// constant instead. A local's name cannot hide anything, because there is
/// nothing in it.
pub(crate) fn alpha_renamed(body: &str) -> String {
  let mut names: Vec<String> = Vec::new();
  let mut out = String::with_capacity(body.len());
  let mut rest = body;
  while let Some(at) = rest.find('%') {
    out.push_str(&rest[..at]);
    rest = &rest[at + 1..];
    let end = rest
      .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '.' || c == '-'))
      .unwrap_or(rest.len());
    let name = &rest[..end];
    // A purely numeric local is already canonical and keeps its own name:
    // renumbering it would renumber it to something else and make two
    // identical bodies differ.
    if name.chars().all(|c| c.is_ascii_digit()) {
      out.push('%');
      out.push_str(name);
    } else {
      let index = match names.iter().position(|seen| seen == name) {
        Some(index) => index,
        None => {
          names.push(name.to_string());
          names.len() - 1
        }
      };
      out.push_str(&format!("%v{index}"));
    }
    rest = &rest[end..];
  }
  out.push_str(rest);
  out
}
