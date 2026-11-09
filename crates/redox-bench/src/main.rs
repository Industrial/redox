//! `redox-bench` — regenerates `benchmarks/REPORT.md`.
//!
//! Plan 44 leaf 2 and §1.4: throughput and latency are **reported, not
//! asserted**. A timing assertion is the flaky gate §1.2 declines, and a
//! threshold would encode this machine's speed into the repository.
//!
//! Criterion 7: the report is regenerable by this command, so a number in it
//! can be checked rather than trusted.

use redox_bench::throughput;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How many times each source is transpiled.
///
/// Enough that the mean is stable to the three significant figures the report
/// prints, and few enough that regenerating it takes seconds. §1.1 measured
/// that no statistics library is available or pinned, and §6 declines to add
/// one: a number reported to three figures does not need one.
const RUNS: u32 = 200;

fn root() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> Result<(), String> {
  let corpus = sources(&root().join("examples"))?;
  let mut rows: Vec<(String, usize, Duration)> = Vec::new();
  let mut total_bytes = 0usize;
  let mut total_time = Duration::ZERO;

  for (name, text) in &corpus {
    let (_, mean) = throughput(text, RUNS);
    total_bytes += text.len();
    total_time += mean;
    rows.push((name.clone(), text.len(), mean));
  }

  // Slowest first: a report sorted by name hides which file costs the most,
  // and that is the only ordering a reader acts on.
  rows.sort_by_key(|r| std::cmp::Reverse(r.2));

  let report = render(&rows, total_bytes, total_time, &machine());
  let path = root().join("benchmarks/REPORT.md");
  std::fs::write(&path, report).map_err(|e| e.to_string())?;
  // `root()` is the manifest directory joined to `../..`, so the path prints
  // as `crates/redox-bench/../../benchmarks/REPORT.md` unless it is resolved.
  // The reader wants the file, not the route taken to it.
  let shown = path.canonicalize().unwrap_or_else(|_| path.clone());
  println!("wrote {}", shown.display());
  Ok(())
}

/// Every `.sdx` in a directory, with its text.
fn sources(dir: &Path) -> Result<Vec<(String, String)>, String> {
  let mut found: Vec<(String, String)> = Vec::new();
  let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
  let mut paths: Vec<PathBuf> = entries
    .filter_map(Result::ok)
    .map(|entry| entry.path())
    .filter(|path| path.extension().is_some_and(|x| x == "sdx"))
    .collect();
  paths.sort();
  for path in paths {
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    // `lexical.sdx` is skipped here, and only here: it does not parse, so
    // transpiling it measures the error path rather than the compiler. Every
    // other sweep in the repository includes it deliberately; this is the one
    // where it would be measuring the wrong thing.
    let name = path
      .file_name()
      .unwrap_or_default()
      .to_string_lossy()
      .into_owned();
    if name == "lexical.sdx" {
      continue;
    }
    found.push((name, text));
  }
  Ok(found)
}

/// What this was measured on. A number without a machine is not a
/// measurement — plan 44 decision 7.
fn machine() -> String {
  let cpu = std::fs::read_to_string("/proc/cpuinfo")
    .ok()
    .and_then(|text| {
      text
        .lines()
        .find(|line| line.starts_with("model name"))
        .and_then(|line| {
          line
            .split_once(": ")
            .map(|(_, name)| name.trim().to_string())
        })
    })
    .unwrap_or_else(|| "unknown CPU".to_string());
  let profile = if cfg!(debug_assertions) {
    "debug"
  } else {
    "release"
  };
  format!("{cpu}, {profile} build")
}

/// The report's text.
fn render(
  rows: &[(String, usize, Duration)],
  total_bytes: usize,
  total_time: Duration,
  machine: &str,
) -> String {
  let mut out = String::new();
  out.push_str(
    "# Benchmarks\n\n\
     Plan 44 (`docs/38 - 44 - Zero-cost Benchmarks.md`) leaf 2. Regenerate with:\n\n\
     ```bash\n\
     cargo run -p redox-bench\n\
     ```\n\n\
     ## What is here, and what is not\n\n\
     **The zero-cost claim is not in this document.** It is asserted in the gate, as \
     identical optimised LLVM IR, by `crates/redox-bench/tests/zero_cost.rs`. §1.2 \
     explains why it is not a timing: a timing assertion is a flaky gate, and three rows \
     of this project have already written *unexplained* into their results because a gate \
     failed for a load-dependent reason.\n\n\
     What *is* here is the measurement that cannot be made exact — how fast transpiling \
     is. It is **reported, not asserted**, for the same reason, and because a threshold \
     would encode one machine's speed into the repository.\n\n",
  );
  out.push_str(&format!("## Machine\n\n{machine}\n\n"));
  out.push_str(&format!(
    "## Methodology\n\n\
     - Each source is transpiled **{RUNS} times**, after one discarded warm run.\n\
     - *Transpile* means parse and emit — not `redox build`, which also writes a file and \
       consults a cache, so timing it would time the filesystem.\n\
     - The figure per file is the **mean**; the total is the sum of the means.\n\
     - Timed with `std::time::Instant`. §1.1 measured that neither `hyperfine` nor \
       `criterion` is available or pinned, and §6 declines to add one: a number reported \
       to three significant figures does not need a statistics library.\n\
     - `examples/lexical.sdx` is excluded, and only here — it does not parse, so \
       transpiling it measures the error path. Every other sweep in the repository \
       includes it deliberately.\n\n"
  ));

  let per_byte = if total_bytes > 0 {
    total_time.as_secs_f64() / total_bytes as f64
  } else {
    0.0
  };
  out.push_str(&format!(
    "## Throughput\n\n\
     {} files, {total_bytes} bytes, {:.3} ms total — **{:.1} MB/s**.\n\n",
    rows.len(),
    total_time.as_secs_f64() * 1000.0,
    if per_byte > 0.0 {
      1.0 / per_byte / 1_000_000.0
    } else {
      0.0
    }
  ));

  out.push_str("| File | Bytes | Mean |\n|---|---:|---:|\n");
  for (name, bytes, mean) in rows {
    out.push_str(&format!(
      "| `{name}` | {bytes} | {:.1} µs |\n",
      mean.as_secs_f64() * 1_000_000.0
    ));
  }

  out.push_str(
    "\n## Watch latency\n\n\
     **Not measured, and the reason is worth more than a number would be.**\n\n\
     Plan 09's watcher rebuilds on a filesystem event, so its latency is \
     `notify`'s delivery plus a transpile plus a write. The transpile is the column \
     above — tens of microseconds — and the other two are the filesystem's, which \
     is not Redox's claim to make.\n\n\
     Measuring the whole thing would produce a number dominated by `notify`'s debounce \
     and the page cache, reported as though it described the compiler. That is the \
     failure mode plan 44 §1.4's *losses reported* discipline exists to prevent, so this \
     section reports the loss instead: **the row did not measure what its register cell \
     asked for**, because the measurement would have been misleading. The successor is \
     whoever wants an end-to-end figure and is willing to name the filesystem in it.\n",
  );
  out
}
