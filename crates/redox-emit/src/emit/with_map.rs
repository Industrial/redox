use crate::emit::items::emit_items;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use redox_ast::Program;
use redox_diag::SourceMap;

/// Emit `program`, and record where each generated line came from.
///
/// Plan 06 (`docs/11 - 06 - Source Map.md`) leaf 2. `source` is the `.sdx`
/// text — needed to turn a span into a line and column — and `source_path` is
/// the name to record in the map's header. Passing `""` for both yields a map
/// whose positions are meaningless, which is exactly what
/// [`crate::emit::emit`] discards.
///
/// This is the real entry point; `emit` delegates to it. It lives in
/// `emit/with_map.rs` rather than beside `emit` because doc 06 §1 permits one
/// construct per file, and §2.3's parent-plus-stem rule is what makes
/// `emit_with_map` the correct name for a file called `with_map.rs` under
/// `emit/`.
pub fn emit_with_map(program: &Program, source: &str, source_path: &str) -> (String, SourceMap) {
  let mut out = Sink::new(source);
  // The items, with the blank-line rule, source order and the gathered test
  // module — all of which a `module` body wants too, which is why they live
  // in `emit_items` rather than here (plan 14).
  emit_items(&mut out, &program.items, 0);

  out.unmark();
  emit_trivia(&mut out, &program.trailing, 0);
  // A source file's final newline arrives here as a trailing `BlankLines`
  // trivium, which would otherwise append a stray blank line to every
  // generated file. Normalising to exactly one terminating newline is a
  // deterministic rule that also guards against other trailing-whitespace
  // drift, which matters because golden tests compare exact bytes.
  while out.ends_with("\n\n") {
    out.pop();
  }
  if !out.is_empty() && !out.ends_with("\n") {
    out.push('\n');
  }
  out.finish(source_path)
}
