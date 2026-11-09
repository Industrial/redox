use crate::emit::expr::emit_expr;
use crate::emit::unparen::unparen;
use redox_ast::FieldInit;

/// A record field's **value** text, with punning expanded.
///
/// `Point { x }` means `Point { x: x }`, so the value of a punned field is
/// the field's own name. The expansion happens here rather than in the parser
/// so the AST records what the author wrote — doc 18 §2.4 notes that field
/// punning "came from Haskell in the first place", which makes it a spelling
/// of the same literal rather than a different one.
///
/// Used by the *update* lowering, where there is no shorthand to fall back
/// on: `p { x }` has to become `__rdx_r.x = x;`.
pub(crate) fn field_value(f: &FieldInit, indent: usize) -> String {
  match &f.value {
    Some(e) => emit_expr(unparen(e), indent),
    None => f.name.text.clone(),
  }
}
