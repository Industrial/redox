use crate::emit::emit_stmts::emit_stmts;
use crate::emit::inline_comment::emit_inline_comment;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::signature::signature;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use redox_ast::FnDecl;

/// Emit a function declaration.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) added the signature; plan 01
/// emitted a bare `fn name()`.
pub(crate) fn emit_fn(out: &mut Sink<'_>, f: &FnDecl) {
  // Leading trivia is the author's comment lines, which belong to no
  // construct's span; attributing them to the declaration would report a
  // comment's line for an error in the code below it.
  out.unmark();
  emit_trivia(out, &f.leading, 0);
  out.mark(f.span);
  out.push_str("fn ");
  out.push_str(&f.name.text);
  out.push_str(&signature(&f.sig));
  out.push_str(" {");
  emit_inline_comment(out, f.trailing.as_ref());
  out.push('\n');
  // A function that declares a return type ends in a tail expression; one
  // that does not returns `()` and ends in a statement. See `emit_stmt`.
  //
  // `emit_stmts` rather than a loop over `emit_stmt`, because doc 18 §3.1's
  // draw arrow lowers to a callback whose body is every statement after it —
  // so the unit of translation is the run.
  emit_stmts(out, &f.body.stmts, RUST_INDENT, f.sig.ret.is_some());
  // The closing brace has no source line — the off-side rule means the author
  // never wrote one. Recording it as synthetic rather than attributing it to
  // the last statement keeps the map from making a claim it cannot support.
  out.unmark();
  out.push_str("}\n");
}
