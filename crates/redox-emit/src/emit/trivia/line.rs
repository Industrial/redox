use crate::emit::pad::pad;
use crate::emit::sink::Sink;
use std::fmt::Write as _;

/// One comment line at `indent`, with `marker` and no trailing space when the
/// comment text is empty.
pub(crate) fn line(out: &mut Sink<'_>, indent: usize, marker: &str, text: &str) {
  pad(out, indent);
  if text.is_empty() {
    out.push_str(marker);
    out.push('\n');
  } else {
    let _ = writeln!(out, "{marker} {text}");
  }
}
