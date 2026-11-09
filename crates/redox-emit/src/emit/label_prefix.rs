/// A loop's or block's label, as the `'name: ` Rust writes before the
/// keyword — or nothing.
///
/// Redox writes `'outer while …` and Rust writes `'outer: while …`, so the
/// colon is the whole of the difference. The trailing space is part of it for
/// the reason `Visibility::rust_text` gives: an unlabelled loop must not
/// acquire a leading space, and the golden corpus compares exact bytes.
pub(crate) fn label_prefix(label: &Option<String>) -> String {
  match label {
    Some(name) => format!("'{name}: "),
    None => String::new(),
  }
}
