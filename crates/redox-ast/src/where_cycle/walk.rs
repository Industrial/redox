/// One step of [`super::where_cycle`]'s depth-first search.
///
/// `state` is `0` unvisited, `1` **on the current path**, `2` finished. The
/// three-state marking is what tells a *cycle* from a vertex merely reached
/// twice: a `2` is a shared dependency and perfectly legal — `c = a + b` with
/// both `a` and `b` using `n` — where a `1` is an edge back into the path
/// being walked, which is a cycle by definition.
///
/// Returns the cycle as the slice of `path` from the revisited vertex onward,
/// so the names reported are the ones actually in the loop and not every
/// vertex visited on the way to it.
///
/// The slice is `path[from..end]` with `end` bound first, where the Rust it
/// replaces wrote `path[from..]`. An **open-ended range has no spelling in
/// Redox** — plan 68 found it here — because `..` demands a right operand
/// and also continues a line, so `from..` at a line end swallows the
/// statement below it.
pub(crate) fn walk(
  at: usize,
  graph: &[Vec<usize>],
  state: &mut [u8],
  path: &mut Vec<usize>,
) -> Option<Vec<usize>> {
  state[at] = 1;
  path.push(at);
  for next in &graph[at] {
    if state[*next] == 1 {
      let from = path.iter().position(|i| *i == *next).unwrap_or(0);
      let end = path.len();
      return Some(path[from..end].to_vec());
    }
    if state[*next] == 0
      && let Some(found) = walk(*next, graph, state, path)
    {
      return Some(found);
    }
  }
  path.pop();
  state[at] = 2;
  None
}
