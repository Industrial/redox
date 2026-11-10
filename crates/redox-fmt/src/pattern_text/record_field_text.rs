use crate::pattern_text::pattern_text;

use redox_ast::RecordPatternField;

/// One field of a record pattern — `x = 0`, or `y` for the shorthand.
///
/// `=`, not Rust's `:`. `RecordPatternField::rust_text` makes the swap in the
/// other direction and its comment says so — *"`=` becomes `:`, which is the
/// one character this form changes"* — which is the whole reason this is a
/// separate printer rather than a reuse.
pub(crate) fn record_field_text(field: &RecordPatternField) -> String {
  match &field.pattern {
    Some(pattern) => format!("{} = {}", field.name.text, pattern_text(pattern)),
    None => field.name.text.clone(),
  }
}
