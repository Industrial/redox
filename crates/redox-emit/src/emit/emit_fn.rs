use crate::emit::inline_comment::emit_inline_comment;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::signature::signature;
use crate::emit::stmt::emit_stmt;
use crate::emit::trivia::emit_trivia;
use redox_ast::FnDecl;

/// Emit a function declaration.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) added the signature; plan 01
/// emitted a bare `fn name()`.
pub(crate) fn emit_fn(out: &mut String, f: &FnDecl) {
  emit_trivia(out, &f.leading, 0);
  out.push_str("fn ");
  out.push_str(&f.name.text);
  out.push_str(&signature(&f.sig));
  out.push_str(" {");
  emit_inline_comment(out, f.trailing.as_ref());
  out.push('\n');
  // A function that declares a return type ends in a tail expression; one
  // that does not returns `()` and ends in a statement. See `emit_stmt`.
  let last = f.body.stmts.len().saturating_sub(1);
  for (i, stmt) in f.body.stmts.iter().enumerate() {
    let tail = f.sig.ret.is_some() && i == last;
    emit_stmt(out, stmt, RUST_INDENT, tail);
  }
  out.push_str("}\n");
}
