/// One line with its `!dbg` attachment removed.
pub(crate) fn strip_debug(line: &str) -> &str {
  match line.find(", !dbg") {
    Some(at) => &line[..at],
    None => line,
  }
}
