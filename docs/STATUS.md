# Status

**Generated.** Every number here is measured from the working tree by
`crates/redox-cli/tests/status.rs` and compared in the gate, so a drifted
number fails the build. Rewrite it with:

```bash
REDOX_BLESS=1 cargo test -p redox-cli --test status
```

which rewrites and then **fails**, so the diff is reviewed rather than
absorbed into a green build.

Plan 47 (`docs/41 - 47 - Status and Declines.md`) built it. The posture is
doc 02's, taken from Emerald: *none of this is rounded up*.

---

## Plans

| Status | Rows |
|---|---:|
| `done` | 61 |
| `superseded` | 4 |
| outstanding | 2 |
| **total** | **67** |

`docs/04 - Redox Plan of Plans.md` is the register and the authority. **`done` is a register status, not a guarantee**: several rows record a later plan finding something they got wrong, which is why the register carries the correction and this table only counts.

---

## Source

| Crate | `src` | `tests` |
|---|---:|---:|
| `redox-ast` | 2876 | 0 |
| `redox-bench` | 588 | 217 |
| `redox-cli` | 1372 | 8436 |
| `redox-diag` | 1741 | 0 |
| `redox-driver` | 2897 | 0 |
| `redox-emit` | 5879 | 4397 |
| `redox-fmt` | 2816 | 1464 |
| `redox-fuzz` | 120 | 268 |
| `redox-lexer` | 2838 | 1146 |
| `redox-lsp` | 893 | 496 |
| `redox-mcp` | 777 | 776 |
| `redox-parser` | 7498 | 608 |
| **total** | **30295** | **17808** |

---

## Corpora

| Corpus | Files | What it asserts |
|---|---:|---|
| `examples/*.sdx` | 35 | the CI-checked source of truth for what works |
| `tests/golden/*.xrs` | 34 | the generated Rust, as exact text |
| `crates/redox-cli/tests/ui/*.sdx` | 41 | the diagnostic a bad program gets |
| `tests/unreachable/*.sdx` | 8 | the gaps below, asserted to **still fail** |
| `benchmarks/pairs/*.sdx` | 9 | generated and hand-written Rust, as identical IR |

**35 examples and 34 goldens is not an error.** `examples/lexical.sdx` deliberately does not parse — it is the error-path example, and `benchmarks/REPORT.md` excludes it from the throughput sweep for the same reason. Reporting "35 examples, all green" would be rounding up by exactly one file.

---

## Gaps

9 Rust constructs a Redox program cannot express. Read out of `docs/GRAMMAR.md`'s `UNREACHABLE` rows rather than listed again here, so the two cannot disagree; each has a fixture that the gate asserts **still fails**, so closing a gap breaks the build until both documents are updated.

| Construct | Fixture |
|---|---|
| Blanket impls | `tests/unreachable/blanket_impl.sdx` |
| Macro definitions | `tests/unreachable/macro_def.sdx` |
| Macro invocations | `tests/unreachable/macro_call.sdx` |
| `as` casts | `tests/unreachable/as_cast.sdx` |
| `extern "C" fn` (export) | `tests/unreachable/extern_fn_abi.sdx` |
| `ref` and `ref mut` | `tests/unreachable/ref_pattern.sdx` |
| `unsafe impl` | `tests/unreachable/unsafe_class.sdx` |
| `unsafe trait` | `tests/unreachable/unsafe_class.sdx` |
| `|` or-patterns | `tests/unreachable/or_pattern.sdx` |

Full grading, with the nearest working alternative for each: `docs/GRAMMAR.md`. Deliberate refusals are a different thing and live in `docs/DECLINED.md`.

---

## What this document does not say

- **No coverage percentage.** `moon run :coverage` produces one. A number here would become a target, and it would mostly measure the emitter's match arms.
- **No test count as a quality measure.** The corpora above are counted because they are corpora; a total would invite reading it as a score.
- **No roadmap and no dates.** What is left is the register's outstanding rows. When they land is not a measurement.
- **No benchmark numbers.** `benchmarks/REPORT.md` has them, and plan 44 §1.2 deliberately keeps them out of the gate because they are timings. Copying one here would make this comparison flake.
