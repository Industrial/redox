use crate::expr_text::argument::argument;
use crate::expr_text::expr_text;
use redox_ast::{Expr, FieldInit};

/// A record literal — `Point { x = 3.0, y = 4.0 }`.
///
/// The field separator is `=` and not `:`, which the corpus settles: `:` would
/// collide with the `::` that ascribes, and `Point { x :: f64 }` is the
/// *declaration*. A field with no value is the shorthand — `Point { x }` —
/// and is printed as the name alone.
///
/// Braces take one space inside, which is again the corpus's habit made a
/// rule. An empty record prints as `Head {}`: there is nothing to put a space
/// around, and `Head { }` would not survive its own idempotence check.
pub(crate) fn record_text(head: &Expr, fields: &[FieldInit], source: &str) -> String {
  if fields.is_empty() {
    return format!("{} {{}}", argument(head, source));
  }
  let printed: Vec<String> = fields
    .iter()
    .map(|field| match &field.value {
      Some(value) => format!("{} = {}", field.name.text, expr_text(value, 0, source)),
      None => field.name.text.clone(),
    })
    .collect();
  format!("{} {{ {} }}", argument(head, source), printed.join(", "))
}
