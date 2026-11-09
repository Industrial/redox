use crate::emit::field_value::field_value;
use redox_ast::FieldInit;

/// One field of a record **literal**: `name: value`, or `name` when punned.
///
/// The punned form is Rust's own shorthand. Doc 18 §2.4's table writes
/// `Point { x: x, y: y }`, which is equivalent — and which clippy's
/// `redundant_field_names` would flag in the user's own build. The doc is
/// corrected rather than followed: a lowering that makes the generated file
/// noisy to lint fails doc 01 §3.4's readability requirement for no gain.
pub(crate) fn one_field(f: &FieldInit, indent: usize) -> String {
  match &f.value {
    Some(_) => format!("{}: {}", f.name.text, field_value(f, indent)),
    None => f.name.text.clone(),
  }
}
