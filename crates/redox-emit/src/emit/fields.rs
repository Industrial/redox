use crate::emit::pad::pad;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::sink::Sink;
use crate::emit::struct_variant_width::STRUCT_VARIANT_WIDTH;
use crate::emit::trivia::emit_trivia;
use redox_ast::ConstructorFields;

/// Emit a constructor's fields, at `indent` spaces.
///
/// `top` distinguishes the two places a constructor's fields appear, which
/// need different punctuation — and, for a record, a different *layout*:
///
/// ```text
/// struct Point {            top: a struct body is always one field per line
///     x: f64,
/// }
///     Click { x: i32 },     a narrow variant body stays on its line
/// struct Marker;            top: a unit needs the `;` that closes an item
///     Close,                a unit variant needs only the comma
/// ```
///
/// The asymmetry is rustfmt's, not a choice: it expands a struct body
/// unconditionally and keeps a *variant* body inline while it fits
/// `struct_variant_width`. Plan 04's guarantee is that the emitter never
/// writes a line rustfmt would re-break, so the emitter has to know both
/// rules rather than pick the tidier-looking one.
pub(crate) fn emit_fields(
  out: &mut Sink<'_>,
  fields: &ConstructorFields,
  indent: usize,
  top: bool,
) {
  match fields {
    ConstructorFields::Unit => {
      out.push_str(if top { ";\n" } else { ",\n" });
    }
    ConstructorFields::Tuple(types) => {
      let texts: Vec<&str> = types.iter().map(|t| t.text.as_str()).collect();
      out.push_str(&format!("({})", texts.join(", ")));
      out.push_str(if top { ";\n" } else { ",\n" });
    }
    ConstructorFields::Record(decls) => {
      // The comma-joined field text, which is what rustfmt's width thresholds
      // are measured against — see `STRUCT_VARIANT_WIDTH`.
      let joined = decls
        .iter()
        .map(|f| format!("{}{}: {}", f.visibility.rust_text(), f.name.text, f.ty.text))
        .collect::<Vec<_>>()
        .join(", ");
      // A field carrying its own doc comment cannot take the one-line form:
      // the comment has to go on a line of its own, and collapsing the record
      // would drop it. Plan 48 leaf 2 found field trivia discarded entirely,
      // which AGENTS.md calls a load-bearing rule — "comments are trivia,
      // never discarded" (doc 02 §10.1) — so the width test now yields to it.
      let documented = decls.iter().any(|f| !f.leading.is_empty());
      if !top && !documented && joined.len() <= STRUCT_VARIANT_WIDTH {
        out.push_str(&format!(" {{ {joined} }},\n"));
        return;
      }

      out.push_str(" {\n");
      for f in decls {
        out.mark(f.span);
        emit_trivia(out, &f.leading, indent + RUST_INDENT);
        pad(out, indent + RUST_INDENT);
        out.push_str(&format!(
          "{}{}: {},\n",
          f.visibility.rust_text(),
          f.name.text,
          f.ty.text
        ));
      }
      out.unmark();
      pad(out, indent);
      out.push_str(if top { "}\n" } else { "},\n" });
    }
  }
}
