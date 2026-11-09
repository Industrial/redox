use crate::emit::emit_clauses::emit_clauses;
use crate::emit::expr::emit_expr;
use crate::emit::pad::pad;
use crate::emit::signature::signature;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use redox_ast::ClassMember;

/// Emit one member of a `trait` or `impl`, at `indent` spaces.
///
/// ```text
/// type Unit                  type Unit;
/// const SIDES :: u32     →   const SIDES: u32;
/// area :: &t -> f64          fn area(&self) -> f64;
/// describe self = "…"        fn describe(&self) -> String { "…" }
/// ```
///
/// **A method with no equations is a required method**, and the whole of the
/// difference is the `;` in place of a body. That is why a class and an
/// instance can share one member emitter: Rust distinguishes what may appear
/// in each, Redox does not, and rustc gives the verdict — the same division
/// of labour AGENTS.md states for every other check.
pub(crate) fn emit_member(out: &mut Sink<'_>, m: &ClassMember, indent: usize) {
  match m {
    ClassMember::AssocType(name) => {
      out.mark(name.span);
      pad(out, indent);
      out.push_str(&format!("type {};\n", name.text));
    }
    ClassMember::AssocConst { name, ty, value } => {
      out.mark(name.span);
      pad(out, indent);
      out.push_str(&format!("const {}: {}", name.text, ty.text));
      if let Some(v) = value {
        out.push_str(&format!(" = {}", emit_expr(v, indent)));
      }
      out.push_str(";\n");
    }
    ClassMember::Method(f) => {
      out.unmark();
      emit_trivia(out, &f.leading, indent);
      out.mark(f.span);
      pad(out, indent);
      let sig = signature(&f.sig, indent);
      out.push_str(&format!("fn {}{sig}", f.name.text));

      if f.clauses.is_empty() {
        out.push_str(";\n");
        return;
      }

      if sig.contains('\n') {
        out.push('\n');
        pad(out, indent);
      } else {
        out.push(' ');
      }
      out.push_str("{\n");
      emit_clauses(out, f, indent);
      out.unmark();
      pad(out, indent);
      out.push_str("}\n");
    }
  }
}
