use crate::emit::emit_stmts::emit_stmts;
use crate::emit::pad_to::pad_to;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::sink::Sink;
use redox_ast::Block;

/// Lower a `try` block — doc 03 §12.4, kept unchanged by doc 18 §8, plan 28.
///
/// ```text
/// result = try                 let result = (|| {
///   a = parse x?                   let a = parse(x)?;
///   Ok a                 →         Ok(a)
///                               })();
/// ```
///
/// # The strongest reachability case in the project
///
/// Doc 03 §12.4: *"Rust has wanted this for years and still has it only
/// unstably; lowering to an immediately-invoked closure gives Redox it on
/// stable Rust today."* Every other row makes an existing Rust construct
/// nicer to spell. This one makes an **unavailable** one available, which is
/// doc 03 §2's fourth admission test answered without qualification.
///
/// The closure's return type is inferred from its tail and its `?`s, so a
/// `try` yielding `Result` and one yielding `Option` both work and neither is
/// annotated. A block whose arms disagree is a rustc error against the block —
/// plan 28 §6, and AGENTS.md's rule.
///
/// # Why this needs its own emitter and `do` does not do it
///
/// Plan 20's `do` emits a *block*, and a block's `?` returns from the
/// enclosing function. The whole of `try` is that it does not — so the
/// difference between the two constructs is exactly the `(|| … )()` wrapper,
/// and nothing else. A `try` lowered to a bare block compiles and its `?`
/// returns from `main`, which is why plan 28 §3 proves this by **running**
/// rather than by text.
pub(crate) fn try_block(body: &Block, indent: usize) -> String {
  // A nested `Sink`, for the reason `emit_do` records: this returns text, and
  // `emit_stmts` writes into a sink. So a `try` block's statements are
  // attributed to the binding that holds it rather than individually — the
  // same coarseness, closed by the same change `emit_do`'s comment names.
  let mut inner = Sink::new("");
  emit_stmts(&mut inner, &body.stmts, indent + RUST_INDENT, true);
  let (statements, _) = inner.finish("");

  let mut out = "(|| {\n".to_string();
  out.push_str(&statements);
  pad_to(&mut out, indent);
  out.push_str("})()");
  out
}
