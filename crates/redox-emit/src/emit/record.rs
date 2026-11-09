use crate::emit::expr::emit_expr;
use crate::emit::field_value::field_value;
use crate::emit::names_a_constructor::names_a_constructor;
use crate::emit::one_field::one_field;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::struct_lit_width::STRUCT_LIT_WIDTH;
use crate::emit::unparen::unparen;
use redox_ast::{Expr, FieldInit};

/// Lower doc 18 §2.4's record literal, starting at column `indent`.
///
/// ```text
/// Point { x = 10, y = 20 }   Point { x: 10, y: 20 }
/// Point { x, y }         →   Point { x, y }
/// p { x = 99 }               { let mut __rdx_r = p; __rdx_r.x = 99; __rdx_r }
/// ```
///
/// **Construction or update is decided by the case of the head**, which doc
/// 18 §5.3 makes meaningful and §2.4 says is "why the update form is
/// admissible at all". An uppercase head names a constructor; anything else
/// names a value.
///
/// Update is one of §6's disclosed lowerings, and §6 gives the reason it is
/// not Rust's `Point { x: 99, ..p }`: that form needs the type's *path*, and
/// the source gives only the value's name. The block needs no type at all,
/// and it moves `p` exactly as `..p` would.
///
/// A punned field emits Rust's own shorthand, `Point { x }`. Doc 18 §2.4's
/// table writes `Point { x: x, y: y }`, which is equivalent — and which
/// clippy's `redundant_field_names` would flag in the user's own build. The
/// doc is corrected rather than followed here: a lowering that makes the
/// generated file noisy to lint fails doc 01 §3.4's readability requirement
/// for no gain.
pub(crate) fn emit_record(head: &Expr, fields: &[FieldInit], indent: usize) -> String {
  let inner_pad = " ".repeat(indent + RUST_INDENT);
  let close_pad = " ".repeat(indent);

  if !names_a_constructor(head) {
    // `p { x = 99 }` — the update lowering. Multi-line, because rustfmt
    // expands a block of statements unconditionally.
    let mut text = format!(
      "{{\n{inner_pad}let mut __rdx_r = {};\n",
      emit_expr(unparen(head), indent + RUST_INDENT)
    );
    for f in fields {
      text.push_str(&format!(
        "{inner_pad}__rdx_r.{} = {};\n",
        f.name.text,
        field_value(f, indent + RUST_INDENT)
      ));
    }
    text.push_str(&format!("{inner_pad}__rdx_r\n{close_pad}}}"));
    return text;
  }

  let head_text = emit_expr(unparen(head), indent);
  if fields.is_empty() {
    return head_text;
  }

  let joined = fields
    .iter()
    .map(|f| one_field(f, indent))
    .collect::<Vec<_>>()
    .join(", ");
  if joined.len() <= STRUCT_LIT_WIDTH && !joined.contains('\n') {
    return format!("{head_text} {{ {joined} }}");
  }

  let mut text = format!("{head_text} {{\n");
  for f in fields {
    text.push_str(&format!(
      "{inner_pad}{},\n",
      one_field(f, indent + RUST_INDENT)
    ));
  }
  text.push_str(&format!("{close_pad}}}"));
  text
}
