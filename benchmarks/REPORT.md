# Benchmarks

Plan 44 (`docs/38 - 44 - Zero-cost Benchmarks.md`) leaf 2. Regenerate with:

```bash
cargo run -p voltaic-bench
```

## What is here, and what is not

**The zero-cost claim is not in this document.** It is asserted in the gate, as identical optimised LLVM IR, by `crates/voltaic-bench/tests/zero_cost.rs`. §1.2 explains why it is not a timing: a timing assertion is a flaky gate, and three rows of this project have already written *unexplained* into their results because a gate failed for a load-dependent reason.

What *is* here is the measurement that cannot be made exact — how fast transpiling is. It is **reported, not asserted**, for the same reason, and because a threshold would encode one machine's speed into the repository.

## Machine

AMD Ryzen 9 7950X 16-Core Processor, debug build

## Methodology

- Each source is transpiled **200 times**, after one discarded warm run.
- *Transpile* means parse and emit — not `voltaic build`, which also writes a file and consults a cache, so timing it would time the filesystem.
- The figure per file is the **mean**; the total is the sum of the means.
- Timed with `std::time::Instant`. §1.1 measured that neither `hyperfine` nor `criterion` is available or pinned, and §6 declines to add one: a number reported to three significant figures does not need a statistics library.
- `examples/lexical.sdx` is excluded, and only here — it does not parse, so transpiling it measures the error path. Every other sweep in the repository includes it deliberately.

## Throughput

27 files, 50616 bytes, 8.919 ms total — **5.7 MB/s**.

| File | Bytes | Mean |
|---|---:|---:|
| `fallible.sdx` | 5511 | 972.9 µs |
| `destructuring.sdx` | 3865 | 893.0 µs |
| `draw_condition.sdx` | 3921 | 702.2 µs |
| `control.sdx` | 3151 | 567.1 µs |
| `strings.sdx` | 3900 | 564.8 µs |
| `async.sdx` | 3323 | 506.8 µs |
| `class.sdx` | 1893 | 393.8 µs |
| `module.sdx` | 2261 | 388.3 µs |
| `arrays.sdx` | 2042 | 386.7 µs |
| `comprehension.sdx` | 1785 | 329.2 µs |
| `section.sdx` | 1782 | 314.6 µs |
| `errors.sdx` | 1221 | 307.5 µs |
| `generic.sdx` | 1882 | 290.9 µs |
| `guards.sdx` | 817 | 251.5 µs |
| `attributes.sdx` | 1902 | 226.4 µs |
| `testing.sdx` | 999 | 215.6 µs |
| `data.sdx` | 1651 | 211.2 µs |
| `visibility.sdx` | 1482 | 208.9 µs |
| `case.sdx` | 886 | 207.1 µs |
| `constants.sdx` | 1815 | 196.5 µs |
| `expressions.sdx` | 740 | 187.5 µs |
| `coalesce.sdx` | 1265 | 168.6 µs |
| `record.sdx` | 733 | 161.6 µs |
| `ascription.sdx` | 1256 | 159.6 µs |
| `phase1.sdx` | 229 | 52.8 µs |
| `comments.sdx` | 239 | 34.4 µs |
| `hello.sdx` | 65 | 19.7 µs |

## Watch latency

**Not measured, and the reason is worth more than a number would be.**

Plan 09's watcher rebuilds on a filesystem event, so its latency is `notify`'s delivery plus a transpile plus a write. The transpile is the column above — tens of microseconds — and the other two are the filesystem's, which is not Voltaic's claim to make.

Measuring the whole thing would produce a number dominated by `notify`'s debounce and the page cache, reported as though it described the compiler. That is the failure mode plan 44 §1.4's *losses reported* discipline exists to prevent, so this section reports the loss instead: **the row did not measure what its register cell asked for**, because the measurement would have been misleading. The successor is whoever wants an end-to-end figure and is willing to name the filesystem in it.
