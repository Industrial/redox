pub(crate) mod mentions;
pub(crate) mod operands;

use crate::where_edges::mentions::mentions;
use crate::{Expr, Stmt};

/// For each entry of a `where` group, the indices of the group's **bindings**
/// its value mentions — plan 61 leaf 1.
///
/// ```text
/// scaled = base * 2.0      [1]     mentions `base`
/// base = w * w             []      `w` is a parameter, not an edge
/// ```
///
/// # Why this lives in `redox-ast` and not in either consumer
///
/// Two crates need the same answer and neither can see the other:
/// `redox-parser` refuses a **cycle** (`RDX0119`) and `redox-emit` sorts into
/// **dependency order**. The first draft of this row wrote the walk twice and
/// the doc comment on the second copy had to say *"must agree with it exactly
/// or the two disagree about whether a program has a cycle"* — which is the
/// one-predicate-two-positions defect plan 34 §10.31 found five times, written
/// down as a rationalisation. One implementation makes them agree by
/// construction.
///
/// It is an accessor over the tree rather than a lowering decision — the same
/// category as [`Expr::blocks`] — which is what keeps AGENTS.md's *"syntax
/// tree types, no logic"* intact. **What** the edges mean is each consumer's;
/// what they *are* is a property of the tree.
///
/// # What is deliberately not an edge
///
/// A reference to a **parameter** or to a **module-scope** name, which is free
/// rather than a check: a name not in the group's own bound set is simply not
/// in the map. `Expr::Path` carries a single string, so `inner::x` is never an
/// `Ident` and never an edge either.
///
/// A **function** entry, on both sides. Nested `fn` items are
/// order-independent in Rust, so a mutually recursive pair needs no ordering
/// and including them would report a cycle on a program that compiles. Since
/// plan 61 leaf 0, anything in a `where` group that takes a parameter *is* a
/// function entry — which is what made leaf 3 free.
///
/// A binding whose pattern defines **several** names — `(a, b) = p`, which
/// plan 61 §11.2 measured — has no single vertex to be, so it is neither a
/// source nor a target. The sort leaves it where it was written, which is the
/// conservative answer and where it already worked.
pub fn where_edges(entries: &[Stmt]) -> Vec<Vec<usize>> {
  let named: Vec<(usize, String)> = entries
    .iter()
    .enumerate()
    .filter_map(|(i, entry)| match &entry.expr {
      Expr::Binding(b) => match b.pattern.binders().as_slice() {
        [one] => Some((i, one.clone())),
        _ => None,
      },
      _ => None,
    })
    .collect();

  entries
    .iter()
    .map(|entry| {
      let Expr::Binding(b) = &entry.expr else {
        return Vec::new();
      };
      let mut found = Vec::new();
      mentions(&b.value, &named, &mut found);
      found.sort_unstable();
      found.dedup();
      found
    })
    .collect()
}
