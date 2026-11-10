use crate::emit::expr::emit_expr;
use crate::emit::pad::pad;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use redox_ast::ConstDecl;

/// Emit doc 03 §13.4's module-scope `const` or `static`, at `indent` spaces.
///
/// ```text
/// const MAX : usize = 1024      const MAX: usize = 1024;
/// static COUNT : u32 = 0    →   static COUNT: u32 = 0;
/// ```
///
/// One keyword apart, which is why one node carries both.
pub(crate) fn emit_const(out: &mut Sink<'_>, c: &ConstDecl, indent: usize) {
  out.unmark();
  emit_trivia(out, &c.leading, indent);
  out.mark(c.span);
  pad(out, indent);
  out.push_str(&format!(
    "{}{} {}: {} = {};\n",
    c.visibility.rust_text(),
    if c.is_static { "static" } else { "const" },
    c.name.text,
    c.ty.text,
    emit_expr(&c.value, indent)
  ));
}
