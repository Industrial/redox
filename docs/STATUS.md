# Status

**Generated.** Every number here is measured from the working tree by
`crates/redox-cli/tests/status.rs` and compared in the gate, so a drifted
number fails the build. Rewrite it with:

```bash
REDOX_UPDATE=1 cargo test -p redox-cli --test status
```

which rewrites and then **fails**, so the diff is reviewed rather than
absorbed into a green build.

Plan 47 (`docs/41 - 47 - Status and Declines.md`) built it. The posture is
doc 02's, taken from Emerald: *none of this is rounded up*.

---

## Plans

| Status | Rows |
|---|---:|
| `done` | 86 |
| `superseded` | 4 |
| outstanding | 0 |
| **total** | **90** |

`docs/04 - Redox Plan of Plans.md` is the register and the authority. **`done` is a register status, not a guarantee**: several rows record a later plan finding something they got wrong, which is why the register carries the correction and this table only counts.

---

## Source

| Crate | `src` | `tests` |
|---|---:|---:|
| `redox-ast` | 3990 | 0 |
| `redox-bench` | 645 | 217 |
| `redox-cli` | 1494 | 9509 |
| `redox-diag` | 1017 | 0 |
| `redox-driver` | 3253 | 0 |
| `redox-emit` | 7594 | 4572 |
| `redox-fmt` | 3149 | 1496 |
| `redox-fuzz` | 142 | 292 |
| `redox-lexer` | 2176 | 1196 |
| `redox-lsp` | 1626 | 756 |
| `redox-mcp` | 884 | 776 |
| `redox-parser` | 10165 | 706 |
| `redox-seed` | 337 | 676 |
| **total** | **36472** | **20196** |

---

## Corpora

| Corpus | Files | What it asserts |
|---|---:|---|
| `examples/*.sdx` | 44 | the CI-checked source of truth for what works |
| `tests/golden/*.xrs` | 43 | the generated Rust, as exact text |
| `crates/redox-cli/tests/ui/*.sdx` | 55 | the diagnostic a bad program gets |
| `tests/unreachable/*.sdx` | 7 | the gaps below, asserted to **still fail** |
| `benchmarks/pairs/*.sdx` | 10 | generated and hand-written Rust, as identical IR |

**44 examples and 43 goldens is not an error.** `examples/lexical.sdx` deliberately does not parse — it is the error-path example, and `benchmarks/REPORT.md` excludes it from the throughput sweep for the same reason. Reporting "44 examples, all green" would be rounding up by exactly one file.

---

## Gaps

8 Rust constructs a Redox program cannot express. Read out of `docs/GRAMMAR.md`'s `UNREACHABLE` rows rather than listed again here, so the two cannot disagree; each has a fixture that the gate asserts **still fails**, so closing a gap breaks the build until both documents are updated.

| Construct | Fixture |
|---|---|
| Macro definitions | `tests/unreachable/macro_def.sdx` |
| Macro invocation with a non-expression argument | `tests/unreachable/macro_item_argument.sdx` |
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
