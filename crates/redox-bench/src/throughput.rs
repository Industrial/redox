pub(crate) mod transpile;

use crate::throughput::transpile::transpile;
use std::time::{Duration, Instant};

/// How long it takes to transpile a source, averaged over `runs`.
///
/// Plan 44 §1.4: **reported, not asserted.** A throughput threshold in the
/// gate would encode this machine's speed into the repository, and a timing
/// assertion is the flaky gate §1.2 declines.
///
/// `Instant` rather than `criterion` or `hyperfine`: §1.1 measured that
/// neither is available or pinned, and §6 declines to add one because the
/// gate's measurement is not a timing and the report's timings are coarse. A
/// number reported to three significant figures does not need a statistics
/// library.
///
/// Returns the total and the per-run mean, so the report can state both. A
/// mean alone hides how long the whole thing took, which is what a reader
/// deciding whether to re-run it wants to know.
pub fn throughput(source: &str, runs: u32) -> (Duration, Duration) {
  // One warm run first, discarded. The first transpile in a process pays for
  // lazy initialisation that has nothing to do with the compiler's speed, and
  // folding it into the mean is the commonest way a benchmark overstates a
  // cost.
  let _ = transpile(source);

  let started = Instant::now();
  for _ in 0..runs {
    let _ = transpile(source);
  }
  let total = started.elapsed();
  let mean = total / runs.max(1);
  (total, mean)
}
