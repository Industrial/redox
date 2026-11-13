# -*- coding: utf-8 -*-
"""Plan 76 batch 3: `main`, and three prose blocks read out of the Rust.

`render` builds `benchmarks/REPORT.md` from three long literals written with
Rust's `\\`-at-end-of-line continuations, so each paragraph is **one line** of
runtime text wrapped across many lines of source. Voltaic has no such
continuation — a string spans lines only as a `\"\"\"` block, and that would put
a real newline at every wrap and hard-wrap the report.

So the three are **extracted from `main.rs` and un-escaped**, not retyped: the
report's text comes out byte-identical, and the port of this file is a
transformation rather than a transcription. `keyword.rs`'s two tables were done
the same way in plan 74 for the same reason.

**And the `as` cast.** `total_time.as_secs_f64() / total_bytes as f64` is this
crate's one blocker in `docs/CONFORMANCE.md`. `u32::try_from` and `f64::from`
express it without one, and the `unwrap_or 0.0` cannot mislead: `per_byte` is
then zero and the `if per_byte > 0.0` guard the Rust already has prints
`0.0 MB/s` — so a corpus somehow larger than 4 GB would report nothing rather
than something wrong.
"""
import io
import re
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

SRC = '/home/tom/Code/rust/voltaic/crates/voltaic-bench/src/main.rs'
text = io.open(SRC, encoding='utf-8').read()


def literal_at(after):
    """The Rust string literal that follows `after`, as its runtime text."""
    # The anchor **includes** the opening quote and the first few characters
    # of the content, so the scan starts just past that quote. Searching for
    # the next quote after the anchor instead found the *closing* one and read
    # off the end of the file.
    at = text.index(after)
    i = at + after.index('"') + 1
    out = []
    while True:
        ch = text[i]
        if ch == '"':
            break
        if ch == '\\':
            nxt = text[i + 1]
            if nxt == '\n':
                # A line continuation: the newline and the following run of
                # spaces are not in the string.
                i += 2
                while text[i] in ' \t':
                    i += 1
                continue
            out.append(ch)
            out.append(nxt)
            i += 2
            continue
        out.append(ch)
        i += 1
    return ''.join(out)


header = literal_at('out.push_str(\n    "# Benchmarks')
methodology = literal_at('"## Methodology')
closer = literal_at('"\\n## Watch latency')

# Each is one Voltaic string literal per `push_str`. `{RUNS}` is Rust's inline
# capture and becomes Voltaic's `#{RUNS}`; nothing else in these three is a
# format argument.
assert '{RUNS}' in methodology, 'the methodology block lost its capture'
methodology = methodology.replace('{RUNS}', '#{RUNS}')
for name, block in (('header', header), ('closer', closer)):
    assert '{' not in block.replace('{-', ''), '%s has a format argument' % name

write('main', '''
-- | How many times each source is transpiled.
-- |
-- | Enough that the mean is stable to the three significant figures the report
-- | prints, and few enough that regenerating it takes seconds. §1.1 measured
-- | that no statistics library is available or pinned, and §6 declines to add
-- | one: a number reported to three figures does not need one.
const RUNS : u32 = 200

-- | The workspace root, from this crate's manifest directory.
root : PathBuf
root = .join "../.." (Path::new (env! \\-> "CARGO_MANIFEST_DIR"))

main : Result<(), String>
main =
  corpus = (sources (&.join "examples" (root ())))?
  mut rows : Vec<(String, usize, Duration)> = Vec::new ()
  mut total_bytes : usize = 0
  mut total_time = Duration::ZERO

  for entry in &corpus
    -- Borrowed: `entry` is a `&(String, String)`, so binding a field by
    -- value is `E0507` — a move out of a shared reference.
    name = &entry.0
    r#text = &entry.1
    measured = throughput r#text RUNS
    mean = measured.1
    total_bytes += .len r#text
    total_time += mean
    rows |> .push ((.clone name, .len r#text, mean))

  -- Slowest first: a report sorted by name hides which file costs the most,
  -- and that is the only ordering a reader acts on.
  .sort_by_key (\\r -> std::cmp::Reverse r.2) rows

  report = render (&rows) total_bytes total_time (&machine ())
  path = .join "benchmarks/REPORT.md" (root ())
  (.map_err (\\e -> .to_string e) (std::fs::write (&path) report))?
  -- `root()` is the manifest directory joined to `../..`, so the path prints
  -- as `crates/voltaic-bench/../../benchmarks/REPORT.md` unless it is resolved.
  -- The reader wants the file, not the route taken to it.
  shown = .unwrap_or (.clone path) (.ok (.canonicalize path))
  println! \\-> "wrote #{(.display shown)}"
  Ok ()

-- | Every `.sdx` in a directory, with its text.
sources : &Path -> Result<Vec<(String, String)>, String>
sources dir =
  mut found : Vec<(String, String)> = Vec::new ()
  shown = .display dir
  entries = (.map_err (\\e -> format! \\-> "#{shown}: #{e}") (std::fs::read_dir dir))?
  mut paths : Vec<PathBuf> = entries
    |> .filter_map Result::ok
    |> .map (\\entry -> .path entry)
    |> .filter (\\path -> .is_some_and (\\x -> x == "sdx") (.extension path))
    |> .collect
  .sort paths
  for path in paths
    r#text = (.map_err (\\e -> .to_string e) (std::fs::read_to_string (&path)))?
    -- `lexical.sdx` is skipped here, and only here: it does not parse, so
    -- transpiling it measures the error path rather than the compiler. Every
    -- other sweep in the repository includes it deliberately; this is the one
    -- where it would be measuring the wrong thing.
    name = path
      |> .file_name
      |> .unwrap_or_default
      |> .to_string_lossy
      |> .into_owned
    if name == "lexical.sdx"
      continue
    found |> .push ((name, r#text))
  Ok found

-- | What this was measured on. A number without a machine is not a
-- | measurement — plan 44 decision 7.
machine : String
machine =
  read = .ok (std::fs::read_to_string "/proc/cpuinfo")
  found = .and_then (\\t -> model_name (&t)) read
  cpu = .unwrap_or (.to_string "unknown CPU") found
  profile = if cfg! \\-> debug_assertions
    "debug"
  else
    "release"
  format! \\-> "#{cpu}, #{profile} build"

-- | The `model name` line of `/proc/cpuinfo`, as its value.
-- |
-- | `#[cfg(test)]` is not wanted here — `machine` calls it — but it is its own
-- | construct because doc 06 §1 allows one per file and `machine`'s closure
-- | chain needed a name to be readable at all.
model_name : &str -> Option<String>
model_name body =
  line = (.find (\\l -> .starts_with "model name" l) (.lines body))?
  split = (.split_once ": " line)?
  Some (.to_string (.trim split.1))

-- | The report's text.
render : &[(String, usize, Duration)] -> usize -> Duration -> &str -> String
render rows total_bytes total_time machine_text =
  mut out = String::new ()
  .push_str HEADER out
  out |> .push_str (&format! \\-> "## Machine\\n\\n#{machine_text}\\n\\n")
  out |> .push_str (&format! \\-> %s)

  -- **`u32::try_from` and `f64::from`, where the Rust wrote `total_bytes as
  -- f64`.** That cast is this crate's one entry in `docs/CONFORMANCE.md`, and
  -- the `unwrap_or 0.0` cannot mislead: `per_byte` is then zero and the guard
  -- below prints `0.0 MB/s`, so a corpus larger than 4 GB reports nothing
  -- rather than something wrong.
  bytes_f = .unwrap_or 0.0 (.map f64::from (.ok (u32::try_from total_bytes)))
  per_byte = if bytes_f > 0.0
    .as_secs_f64 total_time / bytes_f
  else
    0.0
  rate = if per_byte > 0.0
    1.0 / per_byte / 1000000.0
  else
    0.0
  count = .len rows
  ms = .as_secs_f64 total_time * 1000.0
  out |> .push_str (&format! \\-> "## Throughput\\n\\n#{count} files, #{total_bytes} bytes, #{ms:.3} ms total — **#{rate:.1} MB/s**.\\n\\n")

  out |> .push_str "| File | Bytes | Mean |\\n|---|---:|---:|\\n"
  for row in rows
    -- Borrowed, for the reason the loop above records.
    name = &row.0
    bytes = row.1
    us = .as_secs_f64 row.2 * 1000000.0
    out |> .push_str (&format! \\-> "| `#{name}` | #{bytes} | #{us:.1} µs |\\n")

  .push_str CLOSER out
  out

-- | The report's opening, verbatim.
const HEADER : &str = %s

-- | The closing section, verbatim.
const CLOSER : &str = %s
''' % (
    # The template's \`%s\`s run methodology, HEADER, CLOSER — the inline
    # format template comes before the two consts.
    '"' + methodology + '"',
    '"' + header + '"',
    '"' + closer + '"',
))

print('plan 76 batch 3: main (%d + %d + %d chars of prose extracted)'
      % (len(header), len(methodology), len(closer)))
