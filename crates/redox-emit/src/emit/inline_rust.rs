use crate::emit::pad::pad;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use redox_ast::InlineRust;

/// Emit doc 01 §23.1's inline Rust — plan 45 leaf 1.
///
/// The one lowering that is not a lowering: the block's body goes out exactly
/// as it came in, because Redox never parsed it. Everything this function
/// does is about *placing* that text, not changing it.
///
/// Each line is re-indented to the item's level, and the lines' own relative
/// indentation survives — the lexer already stripped the block's common
/// prefix (doc 01 §7.2), so what is left is the shape the author wrote. A
/// blank line is emitted bare rather than padded, so the `.xrs` carries no
/// trailing whitespace for `cargo fmt --check` to object to.
///
/// `out.mark(…)` is called once for the whole block rather than per line.
/// Plan 06's source map therefore points every rustc diagnostic inside the
/// block at the `rust` keyword that introduced it, which is honest: the
/// offsets within the block are the user's own text and Redox has no
/// knowledge of its structure to offer beyond "it came from here".
pub(crate) fn emit_inline_rust(out: &mut Sink<'_>, r: &InlineRust, indent: usize) {
  out.mark(r.span);
  emit_trivia(out, &r.leading, indent);

  for line in r.code.lines() {
    if line.trim().is_empty() {
      out.push('\n');
      continue;
    }
    pad(out, indent);
    out.push_str(line);
    out.push('\n');
  }
}
