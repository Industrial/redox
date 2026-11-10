use crate::emit::items::emit_items;
use crate::emit::pad::pad;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use redox_ast::ModuleDecl;

/// Emit doc 18 §7's `module` as Rust's `mod`, at `indent` spaces.
///
/// ```text
/// module config              mod config;
/// export module parser   →   pub mod parser;
///
/// module tests               mod tests {
///   test "it parses"             #[cfg(test)]
///     assert! \-> 1 == 1              mod __rdx_tests { … }
///                            }
/// ```
///
/// **A bodiless declaration emits `;` and one with a body emits braces**, and
/// those are different Rust: the first loads a sibling file, the second
/// declares the module inline. `ModuleDecl::items` is an `Option` for exactly
/// that reason — an empty `Vec` could not tell `mod name;` from
/// `mod name {}`.
///
/// This is the one item emitter that **recurses**, which plan 14 §6 disclosed
/// as the only part of that plan which is not passthrough. The recursion goes
/// through `emit_items`, so a module body gets the blank-line rule, the
/// source order and its own gathered test module without any of it being
/// written twice.
pub(crate) fn emit_module(out: &mut Sink<'_>, m: &ModuleDecl, indent: usize) {
  out.unmark();
  emit_trivia(out, &m.leading, indent);
  out.mark(m.span);
  pad(out, indent);
  out.push_str(&format!("{}mod {}", m.visibility.rust_text(), m.name.text));

  let Some(items) = &m.items else {
    out.push_str(";\n");
    return;
  };

  out.push_str(" {\n");
  emit_items(out, items, indent + RUST_INDENT);
  out.unmark();
  pad(out, indent);
  out.push_str("}\n");
}
