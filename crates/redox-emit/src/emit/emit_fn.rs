use crate::emit::inline_comment::emit_inline_comment;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::stmt::emit_stmt;
use crate::emit::trivia::emit_trivia;
use redox_ast::FnDecl;

/// Emit a function declaration.
///
/// Parameters and return types are plan 03; this slice emits a bare signature.
pub(crate) fn emit_fn(out: &mut String, f: &FnDecl) {
  emit_trivia(out, &f.leading, 0);
  out.push_str("fn ");
  out.push_str(&f.name.text);
  out.push_str("() {");
  emit_inline_comment(out, f.trailing.as_ref());
  out.push('\n');
  for stmt in &f.body.stmts {
    emit_stmt(out, stmt, RUST_INDENT);
  }
  out.push_str("}\n");
}
