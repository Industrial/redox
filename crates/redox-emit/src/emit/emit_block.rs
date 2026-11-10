use crate::emit::emit_stmts::emit_stmts;
use crate::emit::label_prefix::label_prefix;
use crate::emit::pad_to::pad_to;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::sink::Sink;
use redox_ast::Block;

/// Emit an indented block as a Rust block expression — plan 20, and plan 55
/// leaf 7 for the spelling.
///
/// ```text
/// found = 'search             let found = 'search: {
///   for c in candidates           for c in candidates {
///     …                   →           …
///   None                          }
///                                 None
///                             };
/// ```
///
/// Written `do` and `do 'search` until leaf 7 retired the keyword (doc 03
/// §7.2). The lowering never read it — a block is braces and an optional
/// label — so this function lost its name and nothing else.
///
/// An expression, so this returns text rather than writing into the caller's
/// sink — which means it needs its own `Sink` to run `emit_stmts` into. That
/// is the one place in the emitter where a nested sink is used, and it is why
/// a block's statements are **not** in the source map: the records a nested
/// sink collects are discarded with it.
///
/// Recorded rather than hidden: plan 06's totality test asserts that every
/// *line* is attributed, and a block's lines are attributed to the binding
/// that holds it — which is true but coarse. Closing it needs `emit_expr` to
/// write into a sink instead of returning a `String`, which is a change
/// across every expression and not this plan's.
pub(crate) fn emit_block(label: &Option<String>, body: &Block, indent: usize) -> String {
  let mut inner = Sink::new("");
  emit_stmts(&mut inner, &body.stmts, indent + RUST_INDENT, true);
  let (statements, _) = inner.finish("");

  let mut out = format!("{}{{\n", label_prefix(label));
  out.push_str(&statements);
  pad_to(&mut out, indent);
  out.push('}');
  out
}
