use crate::item_text::visibility_text::visibility_text;

use redox_ast::{Constructor, ConstructorFields};

/// One `data` alternative — `Click`, `Meters f64`, `Square { side : f64 }`.
///
/// The three field shapes are the three Rust shapes with Redox's spellings: a
/// tuple constructor's types are **juxtaposed** rather than parenthesised,
/// because juxtaposition is the application form for types as much as for
/// values; and a record field uses `::`, which is the declaration separator
/// and not the single `:` a local binding takes.
pub(crate) fn constructor_text(constructor: &Constructor) -> String {
  match &constructor.fields {
    ConstructorFields::Unit => constructor.name.text.clone(),
    ConstructorFields::Tuple(types) => {
      let mut out = constructor.name.text.clone();
      for ty in types {
        out.push(' ');
        out.push_str(&ty.text);
      }
      out
    }
    ConstructorFields::Record(fields) => {
      let printed: Vec<String> = fields
        .iter()
        .map(|field| {
          let vis = visibility_text(&field.visibility);
          format!("{}{} : {}", vis, field.name.text, field.ty.text)
        })
        .collect();
      if printed.is_empty() {
        format!("{} {{}}", constructor.name.text)
      } else {
        format!("{} {{ {} }}", constructor.name.text, printed.join(", "))
      }
    }
  }
}
