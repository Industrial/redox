/// The name in a Rust source's first `pub fn NAME(`.
pub(crate) mod first_public_function;

use crate::pair::Pair;
use crate::pair::pairs::first_public_function::first_public_function;
use std::path::Path;

/// Every pair under `benchmarks/pairs/`, sorted by stem.
///
/// A pair is an `.sdx` and an `.rs` sharing a stem, and its **function** is
/// discovered rather than derived from that stem.
///
/// # Why the function's name is not the stem
///
/// It was, and two of the pairs cannot comply: `catch` and `guard` are
/// keywords, so no function can be called either. Worse, the mismatch is
/// **silent in the dangerous direction** — the comparison finds a `define`
/// block by name, so a pair naming its function something else made it look
/// for a symbol that was not there, find an **empty** body, and pass every
/// equality assertion vacuously. `unequal.sdx` did exactly that.
///
/// So the name comes from the hand-written half's first `pub fn`, and the
/// `.sdx` is checked to declare the same one. The convention that matters is
/// *both halves name the same function*, which this enforces; the stem is just
/// a filename.
pub fn pairs(root: &Path) -> Vec<Pair> {
  let dir = root.join("benchmarks/pairs");
  let Ok(entries) = std::fs::read_dir(&dir) else {
    return Vec::new();
  };
  let mut stems: Vec<String> = entries
    .filter_map(Result::ok)
    .map(|entry| entry.path())
    .filter(|path| path.extension().is_some_and(|x| x == "sdx"))
    .filter_map(|path| path.file_stem().map(|s| s.to_string_lossy().into_owned()))
    .collect();
  stems.sort();

  let mut found = Vec::new();
  for stem in stems {
    let sdx = dir.join(format!("{stem}.sdx"));
    let rust = dir.join(format!("{stem}.rs"));
    if !rust.is_file() {
      continue;
    }
    let Some(function) = std::fs::read_to_string(&rust)
      .ok()
      .and_then(|text| first_public_function(&text))
    else {
      continue;
    };
    found.push(Pair {
      stem,
      function,
      sdx,
      rust,
    });
  }
  found
}
