use crate::emit::pad::pad;
use crate::emit::signature::signature;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use redox_ast::ExternBlock;

/// Emit an `extern` block — plan 45 leaf 3.
///
/// ```text
/// extern "C"            extern "C" {
///   abs :: i32 -> i32  →    fn abs(__rdx_a0: i32) -> i32;
/// }
/// ```
///
/// The ABI goes out exactly as written, quotes included: Redox keeps no list
/// of ABIs, so `extern "nonsense"` reaches rustc and rustc rejects it, which
/// is AGENTS.md's every-check-rustc-makes rule.
///
/// Each declaration ends in `;` and has no body — the one place in the
/// emitter where a function signature is written without braces after it.
///
/// # `unsafe` is emitted, not written
///
/// The `unsafe` in `unsafe extern "C" {` is added here and is **not** part of
/// the Redox surface. Edition 2024 made it mandatory — rustc rejects a bare
/// `extern` block with *"extern blocks must be unsafe"*, which is how this was
/// found — so there is no safe form for a user to choose between. Asking them
/// to write a word with exactly one permitted value would be ceremony, and
/// doc 03 §2's *"the user writes which"* rule is about choices, not about
/// constants.
pub(crate) fn emit_extern_block(out: &mut Sink<'_>, e: &ExternBlock, indent: usize) {
  out.mark(e.span);
  emit_trivia(out, &e.leading, indent);
  pad(out, indent);
  out.push_str(&format!("unsafe extern {} {{\n", e.abi));

  // `pad` counts spaces, not levels — a declaration sits one four-space step
  // inside the block.
  let inner = indent + 4;
  for decl in &e.decls {
    out.mark(decl.span);
    emit_trivia(out, &decl.leading, inner);
    pad(out, inner);
    out.push_str(&format!(
      "fn {}{};\n",
      decl.name.text,
      signature(&decl.sig, inner)
    ));
  }

  out.unmark();
  pad(out, indent);
  out.push_str("}\n");
}
