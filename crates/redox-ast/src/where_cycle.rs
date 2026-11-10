/// One step of the dependency search's depth-first walk.
pub(crate) mod walk;

use crate::where_cycle::walk::walk;

use crate::{Stmt, where_edges};

/// The names in one dependency cycle among a `where` group's bindings, if
/// there is one — plan 61 leaf 4.
///
/// ```text
/// a = b + 1      Some(["a", "b"])
/// b = a + 1
/// ```
///
/// A depth-first search with a path stack: the first back edge found names a
/// cycle, and one is all a diagnostic needs. Reporting *every* cycle would be
/// a list the author cannot act on in one edit, and the first one is always
/// real.
///
/// Deterministic because [`where_edges`] sorts each adjacency list and the
/// outer loop walks source order — doc 04's emitter-determinism rule reaching
/// a *diagnostic*, which is the same reason the sort breaks ties by source
/// order.
///
/// A **self** edge counts: `a = a + 1` is a cycle of one and has exactly the
/// same problem.
///
/// # Here rather than in `redox-parser`
///
/// The refusal is the parser's — `RDX0119` — and the search is a structural
/// query over the tree, the same category as [`where_edges`] beside it.
/// Keeping them together is what makes them the *same* graph: the emitter's
/// sort assumes acyclicity because this is what refused one, and two copies
/// of the search could disagree about whether a program has a cycle.
///
/// The depth cap settled where "together" is. `redox-parser`'s own
/// `where_group.rs` sits at `parse/parser/where_group.rs`, which is already
/// the three-component maximum `directory_structure.rs` enforces, so it can
/// have no children — and a recursive search needs a file for its helper.
pub fn where_cycle(entries: &[Stmt]) -> Option<Vec<String>> {
  let graph = where_edges(entries);
  // `repeat_n` where the Rust wrote `vec![0u8; n]`. Plan 64 made macros
  // callable as `name! \-> e`, which reaches `vec!(a, b)` — a two-element
  // vector — and **not** `vec![elem; n]`, a different arm of the macro taking
  // a `;`. `[0u8; n]` is the array-repeat form Redox does have and it needs a
  // const length, which `E0435` says. This is what the macro expands to.
  let mut state: Vec<u8> = std::iter::repeat_n(0u8, entries.len()).collect();
  let mut path = Vec::new();
  let n = entries.len();
  for start in 0..n {
    if state[start] == 0
      && let Some(found) = walk(start, &graph, &mut state, &mut path)
    {
      return Some(
        found
          .iter()
          .filter_map(|i| entries[*i].where_entry_name())
          .collect(),
      );
    }
  }
  None
}
