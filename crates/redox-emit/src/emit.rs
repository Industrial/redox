use crate::emit_fn::emit_fn;
use crate::emit_trivia::emit_trivia;
use redox_ast::{Item, Program};

/// Emit `program` as Rust source text.
pub fn emit(program: &Program) -> String {
  let mut out = String::new();
  for (i, item) in program.items.iter().enumerate() {
    // One blank line between items. Driven by position in the tree, not by
    // how many blank lines the source happened to have, so output is a
    // function of structure and therefore stable.
    if i > 0 {
      out.push('\n');
    }
    match item {
      Item::Fn(f) => emit_fn(&mut out, f),
    }
  }
  emit_trivia(&mut out, &program.trailing, 0);
  // A source file's final newline arrives here as a trailing `BlankLines`
  // trivium, which would otherwise append a stray blank line to every
  // generated file. Normalising to exactly one terminating newline is a
  // deterministic rule that also guards against other trailing-whitespace
  // drift, which matters because golden tests compare exact bytes.
  while out.ends_with("\n\n") {
    out.pop();
  }
  if !out.is_empty() && !out.ends_with('\n') {
    out.push('\n');
  }
  out
}
