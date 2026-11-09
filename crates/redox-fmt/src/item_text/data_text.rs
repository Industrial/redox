use crate::item_text::constructor_text::constructor_text;
use crate::item_text::same_line::same_line;
use crate::item_text::visibility_text::visibility_text;
use crate::stmt_text::indent_text::indent_text;
use redox_ast::DataDecl;

/// A `data` or `newtype` declaration, with its `deriving` clause.
///
/// §3 row 9: one alternative stays on the declaration line and more than one
/// goes one per line with a leading `|`, which is doc 18 §2.4's own example
/// and `examples/data.sdx`'s:
///
/// ```text
/// data Square = Square { side :: f64 }
///
/// data Event
///   = Click
///   | Close
///   deriving (Debug)
/// ```
///
/// The first alternative of a multi-line form takes `=` and the rest take
/// `|`, so the column of operators lines up — which is why the row says
/// *"a leading `|`"* rather than a separator between alternatives.
///
/// `deriving` goes on its own line at one level in, even for the single-line
/// form: `examples/visibility.sdx` writes `export data Point = Point { … }`
/// and then `  deriving (Debug)` below it.
///
/// A `newtype` has exactly one alternative by construction, so it never takes
/// the multi-line shape.
pub(crate) fn data_text(decl: &DataDecl, indent: usize, source: &str) -> String {
  let keyword = if decl.is_newtype { "newtype" } else { "data" };
  let pad = indent_text(indent + 1);
  let header = format!(
    "{}{keyword} {}",
    visibility_text(&decl.visibility),
    decl.name.text
  );

  let mut out = match decl.constructors.as_slice() {
    // One alternative has **two** layouts and the corpus writes both:
    // `data Square = Square { side :: f64 }` in `class.sdx`, and
    //
    // ```text
    // data Profile
    //   = Profile { name :: String, tag :: Tag }
    // ```
    //
    // in `fallible.sdx`. §3 row 9 only distinguishes one alternative from
    // several, so the choice between these is the author's — and the span
    // records it, exactly as it records an equation's inline body.
    [only] if same_line(source, decl.span.0, only.span.0) => {
      format!("{header} = {}", constructor_text(only))
    }
    many => {
      let printed: Vec<String> = many
        .iter()
        .enumerate()
        .map(|(i, constructor)| {
          let lead = if i == 0 { '=' } else { '|' };
          format!("{pad}{lead} {}", constructor_text(constructor))
        })
        .collect();
      format!("{header}\n{}", printed.join("\n"))
    }
  };

  if !decl.derives.is_empty() {
    let names: Vec<&str> = decl.derives.iter().map(|d| d.text.as_str()).collect();
    out.push_str(&format!("\n{pad}deriving ({})", names.join(", ")));
  }
  out
}
