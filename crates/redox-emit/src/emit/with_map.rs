use crate::emit::class::emit_class;
use crate::emit::data::emit_data;
use crate::emit::emit_fn::emit_fn;
use crate::emit::instance::emit_instance;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use crate::emit::type_alias::emit_type_alias;
use redox_ast::{Item, Program};
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
  for (i, item) in program.items.iter().enumerate() {
    // One blank line between items. Driven by position in the tree, not by
    // how many blank lines the source happened to have, so output is a
    // function of structure and therefore stable.
    if i > 0 {
      // The separator belongs to no construct, so it is a synthetic line.
      out.unmark();
      out.push('\n');
    }
    match item {
      Item::Fn(f) => emit_fn(&mut out, f),
      Item::Data(d) => emit_data(&mut out, d),
      Item::Type(a) => emit_type_alias(&mut out, a),
      Item::Class(c) => emit_class(&mut out, c),
      Item::Instance(i) => emit_instance(&mut out, i),
    }
  }
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
