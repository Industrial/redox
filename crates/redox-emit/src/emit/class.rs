use crate::emit::member::emit_member;
use crate::emit::pad::pad;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use redox_ast::{ClassDecl, TypeApp};

/// Emit doc 18 §2.5's `class` as a Rust `trait`.
///
/// ```text
/// class Debug => Shape         trait Shape: Debug {
///   type Unit                      type Unit;
///   const SIDES : u32         →   const SIDES: u32;
///   area : &self -> f64           fn area(&self) -> f64;
///                              }
///
/// class Convert t              trait Convert<T> {
///   wrap : t -> Self          →   fn wrap(__rdx_a0: T) -> Self;
///                              }
/// ```
///
/// The class's parameters become the trait's generic list, **in source
/// order** — plan 67 leaf 2, and the order is the source's because the
/// emitter's output has to be byte-stable. The superclass list becomes Rust's
/// supertrait bounds, joined with `+`. Nothing else changes: Rust's trait
/// system *is* Haskell's class system, so doc 18 §2.5 changes the words and
/// leaves the content alone.
pub(crate) fn emit_class(out: &mut Sink<'_>, c: &ClassDecl, indent: usize) {
  out.unmark();
  emit_trivia(out, &c.leading, indent);
  out.mark(c.span);

  pad(out, indent);
  out.push_str(&format!(
    "{}trait {}",
    c.visibility.rust_text(),
    c.name.text
  ));
  if !c.params.is_empty() {
    let params: Vec<String> = c
      .params
      .iter()
      .map(|p| TypeApp::rust_name(&p.text))
      .collect();
    out.push_str(&format!("<{}>", params.join(", ")));
  }
  if !c.supers.is_empty() {
    let names: Vec<&str> = c.supers.iter().map(|s| s.text.as_str()).collect();
    out.push_str(&format!(": {}", names.join(" + ")));
  }
  out.push_str(" {\n");

  for m in &c.members {
    emit_member(out, m, indent + RUST_INDENT);
  }

  out.unmark();
  pad(out, indent);
  out.push_str("}\n");
}
