/// Post-order depth-first, which is what makes the result a topological order:
/// an entry is pushed only once everything it mentions has been.
pub(crate) fn visit(
  at: usize,
  graph: &[Vec<usize>],
  done: &mut [bool],
  visiting: &mut [bool],
  sorted: &mut Vec<usize>,
) {
  if done[at] || visiting[at] {
    return;
  }
  visiting[at] = true;
  for &dep in &graph[at] {
    visit(dep, graph, done, visiting, sorted);
  }
  visiting[at] = false;
  done[at] = true;
  sorted.push(at);
}
