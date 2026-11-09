use crate::emit::generics::generics;
use crate::emit::member::emit_member;
use crate::emit::pad::pad;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use redox_ast::InstanceDecl;

/// Emit doc 18 §2.5's `instance` as a Rust `impl`.
///
/// ```text
/// instance Display t => Describe (Wrapper t)   impl<T: Display> Describe for Wrapper<T> {
/// instance Counter                         →   impl Counter {
/// ```
///
/// The generic list is **synthesised** — doc 18 §5.3's "type variables need
/// no declaration … the generic list is synthesised from first appearance" —
/// and the constraints become its bounds. That is the one place a Redox type
/// is assembled rather than copied verbatim, and `TypeApp` exists to make it
/// possible: a captured string cannot say which of its words were variables.
pub(crate) fn emit_instance(out: &mut Sink<'_>, i: &InstanceDecl, indent: usize) {
  out.unmark();
  emit_trivia(out, &i.leading, indent);
  out.mark(i.span);

  pad(out, indent);
  out.push_str("impl");
  out.push_str(&generics(&i.target, &i.constraints));
  out.push(' ');
  if let Some(class) = &i.class {
    out.push_str(&format!("{} for ", class.text));
  }
  out.push_str(&i.target.rust_text());
  out.push_str(" {\n");

  for m in &i.members {
    emit_member(out, m, indent + RUST_INDENT);
  }

  out.unmark();
  pad(out, indent);
  out.push_str("}\n");
}
