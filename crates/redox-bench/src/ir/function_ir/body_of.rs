use crate::ir::function_ir::strip_debug::strip_debug;

/// The `define` block whose symbol mentions `name`, with line-level noise
/// dropped.
///
/// Dropped: `!dbg` attachments, the metadata block, and leading whitespace.
/// Those differ by input *file name* rather than by generated code, and the
/// file names are necessarily different between the two sides of a pair.
///
/// Kept: everything else, including instruction order and attributes. A
/// comparison that normalised more than this would be the post-hoc filter
/// decision 5 declined.
pub(crate) fn body_of(text: &str, name: &str) -> String {
  let mut out = String::new();
  let mut inside = false;
  for line in text.lines() {
    if line.starts_with("define") && line.contains(name) {
      inside = true;
      // The `define` line itself carries the mangled symbol, which includes a
      // hash of the source path. Recorded as a marker rather than compared.
      out.push_str("define <symbol>\n");
      continue;
    }
    if inside {
      if line == "}" {
        break;
      }
      let trimmed = line.trim();
      if trimmed.starts_with('!') {
        continue;
      }
      out.push_str(strip_debug(trimmed));
      out.push('\n');
    }
  }
  out
}
