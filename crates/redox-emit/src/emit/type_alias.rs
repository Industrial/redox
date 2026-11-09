use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use redox_ast::TypeAlias;

/// Emit doc 18 §2.4's `type` alias, which is already Rust.
///
/// The keyword, the `=` and the type are unchanged, and the type is written
/// verbatim (doc 03 §14.1) — so this is the shortest lowering in the emitter
/// and the one with the least to go wrong.
pub(crate) fn emit_type_alias(out: &mut Sink<'_>, a: &TypeAlias) {
  out.mark(a.span);
  emit_trivia(out, &a.leading, 0);
  out.push_str(&format!(
    "{}type {} = {};\n",
    a.visibility.rust_text(),
    a.name.text,
    a.ty.text
  ));
}
