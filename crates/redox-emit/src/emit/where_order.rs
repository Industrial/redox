pub(crate) mod visit;

use crate::emit::where_order::visit::visit;
use redox_ast::{Expr, Stmt, where_edges};

/// A `where` group's entries in **dependency order** — plan 61 leaf 2.
///
/// ```text
/// area w = scaled              fn area(w: f64) -> f64 {
///   where                          let base = w * w;
///     scaled = base * 2.0          let scaled = base * 2.0;
///     base = w * w                 scaled
///                              }
/// ```
///
/// The source order reverses and the output does not. Doc 03 §8.3 refused
/// this — *"synthesising a correct order would require dependency analysis
/// this surface does not admit"* — and plan 61 §1.2 is the argument that it
/// refused the wrong thing: the analysis §8.3 declines asks *what does this
/// name mean*, and this asks only *which of these names appears in that body*,
/// over a set known in advance. No scope, no types, no inference.
///
/// # Three properties, each load-bearing
///
/// **Stable.** Ties break by source order, so the output is deterministic —
/// doc 04's emitter-determinism rule, which a sort could quietly violate. The
/// four `where` clauses that were in the corpus before this row are preserved
/// byte for byte by that alone.
///
/// **Functions do not move.** Since leaf 0 a group holds two kinds of entry,
/// and only the bindings are `let`s whose order matters: nested `fn` items are
/// order-independent in Rust, verified in either definition order, so a
/// mutually recursive pair needs no ordering at all. Each non-binding entry
/// keeps its index and the bindings fill the remaining slots in sorted order —
/// simpler than sorting a mixed group, and what made leaf 3 free.
///
/// **It never rewrites `Program`.** `redox-fmt` and `redox-emit` are two
/// readers of one tree, so a pass that mutates the tree to serve one silently
/// changes what the other produces — here, a *formatter* reordering lines its
/// author wrote, which is an edit and not a formatting change. Plan 56 §11.3
/// reaches the same rule from the other side.
///
/// # A cycle cannot reach here
///
/// `where_group` refuses one in the parser (`RDX0119`), using
/// [`where_edges`] — the **same** graph this sorts, which is why the two
/// cannot disagree about whether a program has a cycle. The `visiting` guard
/// is kept anyway and returns rather than recursing: a parse error that
/// somehow reached the emitter would otherwise be an infinite recursion, and
/// the emitter must not be the thing that hangs.
pub(crate) fn where_order(entries: &[Stmt]) -> Vec<usize> {
  let bindings: Vec<usize> = entries
    .iter()
    .enumerate()
    .filter(|(_, e)| matches!(e.expr, Expr::Binding(_)))
    .map(|(i, _)| i)
    .collect();
  if bindings.len() < 2 {
    return (0..entries.len()).collect();
  }

  let graph = where_edges(entries);
  let mut done = vec![false; entries.len()];
  let mut visiting = vec![false; entries.len()];
  let mut sorted = Vec::with_capacity(bindings.len());
  for &i in &bindings {
    visit(i, &graph, &mut done, &mut visiting, &mut sorted);
  }

  // Each non-binding entry keeps its slot; the bindings fill the rest in
  // sorted order.
  let mut sorted = sorted.into_iter();
  (0..entries.len())
    .map(|i| {
      if matches!(entries[i].expr, Expr::Binding(_)) {
        sorted.next().unwrap_or(i)
      } else {
        i
      }
    })
    .collect()
}
