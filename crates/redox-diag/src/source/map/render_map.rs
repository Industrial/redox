use crate::source::map::SourceMap;
use crate::source::map::version::VERSION;
use std::fmt::Write as _;

/// Serialise a map as the `.xrs.map` sidecar.
///
/// Plan 06 (`docs/11 - 06 - Source Map.md`) leaf 4. Text, one record per
/// generated line:
///
/// ```text
/// redox-map 1 examples/phase1.sdx
/// 1 1 1 4
/// 2 2 3 5
/// 3 -
/// ```
///
/// `gen_line src_line src_col gen_col`, or `gen_line -` for a synthetic line.
/// The generated line number is written even though it is the record's index,
/// because the first thing anyone does with this file is open it next to the
/// `.xrs` and compare a line — and counting lines in a text editor to find
/// record 47 is exactly the friction that stops people checking.
pub fn render_map(map: &SourceMap) -> String {
  let mut out = format!("redox-map {VERSION} {}\n", map.source_path);
  for gen_line in 1..=map.len() {
    match map.origin(gen_line) {
      Some(o) => {
        let _ = writeln!(out, "{gen_line} {} {} {}", o.src_line, o.src_col, o.gen_col);
      }
      None => {
        let _ = writeln!(out, "{gen_line} -");
      }
    }
  }
  out
}
