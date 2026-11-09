use crate::emit::emit_clauses::emit_clauses;
use crate::emit::inline_comment::emit_inline_comment;
use crate::emit::pad::pad;
use crate::emit::signature::signature;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use redox_ast::FnDecl;

/// Emit a function declaration.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) added the signature; plan 01
/// emitted a bare `fn name()`.
pub(crate) fn emit_fn(out: &mut Sink<'_>, f: &FnDecl, indent: usize) {
  // Leading trivia is the author's comment lines, which belong to no
  // construct's span; attributing them to the declaration would report a
  // comment's line for an error in the code below it.
  out.unmark();
  emit_trivia(out, &f.leading, indent);
  out.mark(f.span);
  pad(out, indent);
  out.push_str(&f.visibility.rust_text());
  if f.sig.is_const {
    out.push_str("const ");
  }
  // Doc 03 §14.3's `async fn` — plan 39. Both markers are written if both are
  // present; Rust has no `const async fn` and rustc is what says so.
  if f.sig.is_async {
    out.push_str("async ");
  }
  out.push_str("fn ");
  out.push_str(&f.name.text);
  let sig = signature(&f.sig, indent);
  out.push_str(&sig);
  // A `where` clause ends a line, so the body's brace goes on the next one —
  // which is what rustfmt does and the only form it leaves alone.
  if sig.contains('\n') {
    out.push('\n');
    pad(out, indent);
    out.push('{');
  } else {
    out.push_str(" {");
  }
  emit_inline_comment(out, f.trailing.as_ref());
  out.push('\n');
  // A function that declares a return type ends in a tail expression; one
  // that does not returns `()` and ends in a statement. See `emit_stmt`.
  //
  // `emit_clauses` rather than the body directly, because doc 18 §2.3 makes a
  // function one *or more* equations: several clauses, or one refutable
  // pattern, lower to a `match` over the parameters.
  emit_clauses(out, f, indent);
  // The closing brace has no source line — the off-side rule means the author
  // never wrote one. Recording it as synthetic rather than attributing it to
  // the last statement keeps the map from making a claim it cannot support.
  out.unmark();
  pad(out, indent);
  out.push_str("}\n");
}
