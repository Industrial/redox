use crate::emit::pad::pad;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use redox_ast::ImportDecl;

/// Emit doc 03 §13.1's `import` as a Rust `use`, at `indent` spaces.
///
/// ```text
/// import std::collections::HashMap          use std::collections::HashMap;
/// import std::io::{self, Read, Write}   →   use std::io::{self, Read, Write};
/// import std::collections::HashMap as Map   use std::collections::HashMap as Map;
/// export import crate::models::User         pub use crate::models::User;
/// ```
///
/// The shortest lowering in the emitter: a keyword, a verbatim path, and a
/// semicolon. `export import` is `pub use` because `visibility` is read
/// before the dispatch, so the re-export form costs nothing here.
pub(crate) fn emit_import(out: &mut Sink<'_>, i: &ImportDecl, indent: usize) {
  out.unmark();
  emit_trivia(out, &i.leading, indent);
  out.mark(i.span);
  pad(out, indent);
  out.push_str(&format!("{}use {}", i.visibility.rust_text(), i.path));
  if let Some(alias) = &i.alias {
    out.push_str(&format!(" as {}", alias.text));
  }
  out.push_str(";\n");
}
