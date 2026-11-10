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
/// data Square = Square { side : f64 }
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
/// A `newtype` has exactly one alternative, so it never takes the multi-line
/// shape.
///
/// "By construction" is what this said, and it was not true: nothing
/// validated a `newtype`'s shape at all, and `newtype Multi = A | B` parsed
/// and emitted an `enum`. Plan 53 leaf 2 is what makes the invariant real —
/// the parser now refuses a `newtype` with anything other than one
/// constructor over one field — so this printer may rely on it, and could
/// not before.
pub(crate) fn data_text(decl: &DataDecl, indent: usize, source: &str) -> String {
  // Plan 45 leaf 4 added a third keyword to this family. Getting it wrong
  // here is not a cosmetic bug: the formatter wrote `union Word` back as
  // `data Word`, which turns a union into a struct, and
  // `formatting_preserves_meaning` is the test that caught it.
  let keyword = if decl.is_union {
    "union"
  } else if decl.is_newtype {
    "newtype"
  } else {
    "data"
  };
  let pad = indent_text(indent + 1);
  let header = format!(
    "{}{} {}",
    visibility_text(&decl.visibility),
    keyword,
    decl.name.text
  );
  let mut out = match decl.constructors.as_slice() {
    [only] if same_line(source, decl.span.0, only.span.0) => {
      format!("{} = {}", header, constructor_text(only))
    }
    many => {
      // Two things the Rust wrote differently, both forced.
      //
      // One parameter and two bindings, where the Rust wrote the pattern
      // `|(i, constructor)|`: a **lambda parameter is a name, not a
      // pattern**, so `\(i, constructor) ->` earns `RDX0103`. Plan 69's
      // third finding, and the one with the widest reach — every
      // `.enumerate()` and every `.zip(…)` meets it —
      // `tests/ui/lambda_parameter_is_not_a_pattern` is the fixture.
      //
      // And this comment sits above the binding rather than beside the stage
      // it describes, because **a comment between two pipeline stages breaks
      // the pipeline** — `RDX0102` on the `|>` after it, probe
      // `tests/ui/comment_between_pipeline_stages`, which is graded a
      // **defect** rather than a decision: comments are trivia, and
      // `AGENTS.md` says trivia is never discarded.
      let printed: Vec<String> = many
        .iter()
        .enumerate()
        .map(|pair| {
          let i = pair.0;
          let constructor = pair.1;
          let lead = if i == 0 { '=' } else { '|' };
          format!("{}{} {}", pad, lead, constructor_text(constructor))
        })
        .collect();
      format!("{}\n{}", header, printed.join("\n"))
    }
  };
  if !decl.derives.is_empty() {
    let names: Vec<&str> = decl.derives.iter().map(|d| d.text.as_str()).collect();
    out.push_str(&format!("\n{}deriving ({})", pad, names.join(", ")));
  }
  out
}
