# Plan 44 — Zero-cost Benchmarks

Plan ID: `zero-cost-benchmarks`
Depends on: 09 (`watch-mode`)
Milestone: 7 — *tooling*
Status: `done` 2026-11-09

---

## 1. What this plan is

Doc 03's zero-cost claim is *"a claim until measured"*, as doc 04's row says. This row measures
it — and §1.2 is a correction to **how** the row proposed to.

Three things, in two categories:

| What | How | Where |
|---|---|---|
| Generated versus hand-written Rust | **identical optimised IR**, not a timing comparison | asserted in the gate |
| Transpile throughput | timed, N runs, methodology disclosed | reported in `benchmarks/REPORT.md` |
| Watch latency | likewise | likewise |

### 1.1 Verified preconditions

Measured, 2026-11-09.

**There is no benchmarking tool here.** No `hyperfine`, and `criterion` is not in `Cargo.lock`.
Neither is pinned. §1.4 is why that matters less than it would for a timing-first design.

**rustc is locatable.** Plan 07's build script bakes `VOLTAIC_RUSTC` in after verifying a
candidate answers `--version` like real rustc, which AGENTS.md records as the only trustworthy
way to find it here. The same path serves this row.

**And the central claim already holds in two cases**, spiked before this plan was written
because a row whose whole subject is a measurement should not be planned on an assumption.

`o ?? 0` generates a `match`, where a Rust author writes `o.unwrap_or(0)`:

```
fn pick(o: Option<i32>) -> i32 {        fn pick(o: Option<i32>) -> i32 {
    match o {                               o.unwrap_or(0)
        Some(__rdx_v) => __rdx_v,       }
        None => 0,
    }
}
```

Compiled `-O --emit=llvm-ir` under the same crate name, the `pick` bodies are **identical**.
So is the pair for a section inside a pipeline — `xs |> into_iter |> filter (> 1) |> sum`
against `xs.into_iter().filter(|n| *n > 1).sum()`, where the generated closure is
`|__rdx_1| __rdx_1.gt(&1)` and the hand-written one is `|n| *n > 1`.

Two of the row's hardest cases, proven exactly, before any harness existed.

### 1.2 "Within noise" is the wrong assertion, and the register should say so

Doc 04's row 44 says *"compiled identically, asserted within noise"*. That is a **timing**
comparison, and this row declines it for the gate.

A timing assertion is a flaky gate. Plan 42 §1.3 refused to build one and gave the reason:
plan 23 §10.8, doc 30 §8.7 and plan 43 §10.3 are three rows that wrote *unexplained* into their
results because a gate failed for a load-dependent reason. This session alone has seen the
gate's blaze targets SIGKILLed by memory pressure from outside the repository. A benchmark
asserting "within noise" on that machine would fail for the machine's reasons and be believed
about the compiler's.

**Compare the optimised IR instead.** It is:

- **deterministic** — the same bytes every run, so the gate never flakes;
- **exact** — "within noise" becomes "identical", which is the claim doc 03 actually makes;
- **actionable on failure** — a timing regression says *slower*, an IR diff says *what was
  added*.

The one thing it is not is a *runtime* measurement, and §6 records that honestly: identical IR
is sufficient for zero cost and a timing comparison is not necessary once you have it. The
direction that matters — proving no cost was added — is the direction IR equality answers.

### 1.3 Only the lowerings that generate more than the author wrote

Most Voltaic lowerings are **syntactic**: `x |> f` becomes `x.f()`, `\n -> n + 1` becomes
`|n| n + 1`, `until c` becomes `while !(c)`. For those, the generated Rust *is* the Rust a
human writes, so the zero-cost claim is trivially true at the source level and measuring it
measures rustc.

The claim has content where the emitter generates **more** than the author wrote:

| Lowering | What appears |
|---|---|
| `??` and `catch` | a `match` where a human writes `unwrap_or` / `unwrap_or_else` |
| `?.` | a chain of `and_then`, per doc 03 §12.3 |
| a section — `(> 0)`, `(.name)` | a closure, with the `gt(&…)` method form doc 18 §4 requires |
| `with` | a closure and a callback |
| a comprehension | an iterator chain with `filter_map` |
| `guard` | a `let … else` |
| `try` and `async` blocks | a block expression |

Those are the pairs. N programs, each a `.sdx` and the `.rs` a competent Rust author would
write for the same job — which is the `benchmarks/REPORT.md` discipline doc 04 names, with the
pairing made explicit rather than left to a reader's judgement.

### 1.4 Throughput and latency are reported, not asserted

Both are timings, so both are out of the gate by §1.2's argument. They are still worth having,
and the Emerald discipline doc 04 names is exactly how: *N programs, N runs, methodology
disclosed, losses reported*.

So `benchmarks/REPORT.md` is a committed document with numbers in it, produced by a command
anyone can re-run, and it says what machine it was measured on. A number without a machine is
not a measurement.

**Losses reported** is the part that makes it honest. If transpiling is slower than it should
be, or if watch latency is worse than the claim, the report says so — a benchmark document that
only contains wins is marketing.

---

## 2. Admission tests (doc 03 §2)

Not applicable: no construct, no lowering.

---

## 3. Concrete proof this plan targets

```bash
moon run :test                      # the IR comparison, per pair
cargo run -p voltaic-bench -- report  # regenerates benchmarks/REPORT.md
```

The gate asserts, for every pair:

1. Both sides compile under `-O` with no diagnostics.
2. The **named function's** optimised IR is byte-identical between them, after the crate name
   is held constant so symbol mangling agrees.

**Plus a negative case**, because an IR comparison that passes on every pair cannot
distinguish *the lowering is free* from *the comparison is vacuous*: a pair whose hand-written
side is deliberately **not** equivalent — an extra clone — asserted to produce a **different**
IR. If that passes too, the harness is comparing nothing.

---

## 4. Leaves

### Leaf 1 — `leaf-ir-equality`

`crates/voltaic-bench/` with the IR comparison, the pair corpus under `benchmarks/pairs/`, and
the negative case. This is the leaf that carries doc 03's claim.

### Leaf 2 — `leaf-report`

The throughput and watch-latency measurements, and `benchmarks/REPORT.md` — with the machine
named and losses reported.

---

## 5. Acceptance criteria

1. One pair per row of §1.3's table, each a `.sdx` and a hand-written `.rs`.
2. Both sides of every pair compile under `-O` with no diagnostics.
3. The named function's optimised IR is identical for every pair.
4. A deliberately non-equivalent pair produces a **different** IR, so criterion 3 is not
   vacuous.
5. The comparison is deterministic: the same verdict on every run, with no timing anywhere in
   the gate.
6. `benchmarks/REPORT.md` carries transpile throughput and watch latency, the methodology, the
   machine, and any loss.
7. The report is **regenerable** by a committed command, so a number in it can be checked
   rather than trusted.

---

## 6. Out of scope, and what it discloses

- **A runtime comparison in the gate.** §1.2. Identical IR is sufficient for zero cost; a
  timing comparison is not necessary once you have it, and it is a flaky gate.
- **Asserting throughput or latency.** §1.4: reported, because the same argument applies and
  because a throughput *threshold* would encode this machine's speed into the repository.
- **Pinning `hyperfine` or adding `criterion`.** Neither is needed: the gate's measurement is
  not a timing, and the report's timings are coarse enough for `Instant`. Plan 42 §6 declined a
  toolchain addition on the same grounds and plan 41 leaf 1 accepted one — the difference is
  whether a *committed artefact* depends on the tool's version, and here none does.
- **Comparing machine code rather than IR.** Assembly is more faithful and less portable: it
  differs by target and by rustc's register allocator in ways that are not a cost. IR after
  optimisation is the level where "the same program" is checkable.
- **Measuring compile time of the generated Rust.** A real question — generated code that
  compiles slowly is a cost — and a different one from doc 03's claim, which is about the
  produced binary. Named rather than folded in.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

`benchmarks/REPORT.md`'s methodology discipline, which doc 04 names: N programs, N runs,
methodology disclosed, losses reported. §1.4 adopts it for the timings and §1.3 extends it to
the pairs, where "N programs" becomes "one pair per lowering that generates more than the
author wrote".

**Not taken**: Emerald's benchmarks were timings throughout. §1.2 is this row's departure, and
it is a departure from doc 04's own cell as much as from Emerald.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | Assert **identical optimised IR** | "Within noise" timing, as the register says | Deterministic, exact, and actionable on failure; a timing gate is the flake plan 42 §1.3 refused and three rows have already paid for |
| 2 | Only the lowerings that generate more than the author wrote | A pair per construct | For a syntactic lowering the generated Rust *is* what a human writes, so the comparison measures rustc |
| 3 | Report throughput and latency, do not assert them | A threshold in the gate | A threshold encodes this machine's speed into the repository, and §1.2's argument applies unchanged |
| 4 | Compare IR, not assembly | Assembly | Assembly differs by target and register allocation in ways that are not a cost |
| 5 | Hold the crate name constant across both sides | Normalise symbol names after the fact | Symbol mangling includes the crate name; fixing it is one flag, and post-hoc normalisation is a filter that can hide a real difference |
| 6 | A deliberately unequal pair as the negative case | Only the equal pairs | An all-passing comparison cannot distinguish *free* from *vacuous* — plan 43 §10.1 and plan 41 criterion 4's discipline |
| 7 | Name the machine in the report | Numbers alone | A number without a machine is not a measurement |

---

## 10. Result

Shipped 2026-11-09. `crates/voltaic-bench/` with the IR comparison, **nine pairs** under
`benchmarks/pairs/` — eight equal and one deliberately unequal — and `benchmarks/REPORT.md`
regenerated by `cargo run -p voltaic-bench`.

Doc 03's zero-cost claim now holds as an **asserted, deterministic** property of every
lowering that generates more than the author wrote. §1.2's argument survived contact: there is
no timing anywhere in the gate, and the comparison has not flaked once.

### 10.1 Three ways the comparison was vacuous before it was right

The negative pair (§3) was the plan's best decision. It is not what caught these, but it is
why they were looked for.

**An rlib at `-O` does not codegen a `pub fn` at all.** rustc stores it as MIR for a
downstream crate to inline, and `--emit=llvm-ir` produces a module with no `define` for it.
*Both* halves came back empty, and two empty bodies compare equal — a passing gate asserting
nothing. Fixed by injecting `#[no_mangle]` and `#[inline(never)]` onto the compared function
so rustc must emit it whole, and by asserting the extracted body is non-empty.

That fix also **simplified decision 5**: `no_mangle` makes the symbol the bare function name,
with no crate name or path hash, so holding `--crate-name` constant stopped being what made
the two sides comparable. The flag stays because it costs nothing and keeps the reasoning true
if the marking ever changes.

**`catch` differed only in SSA local names** — `%__rdx_v` against `%value` — which is the
generated identifier prefix leaking into the comparison, not a cost. Locals are now
alpha-renamed by first appearance.

**`unequal`'s function was named `first` while the pair's stem was `unequal`.** The comparison
looked for a symbol named after the file, found none, and passed — the *negative* case passing
vacuously, which would have disarmed the one check that detects vacuity. The target name is
now discovered from the hand-written half's first `pub fn`, which is also necessary because
`catch` and `guard` are keywords and cannot name a function.

### 10.2 §1.3's table was wrong about `with`, and wrong to pair `try` with `async`

Both corrections were found by trying to write the pairs the table asked for.

**`with` and `try` had no pair at all** when the implementation was first written — six of the
table's rows were covered and two were not. Both are now covered, which is what criterion 1
actually asks for.

**`async` is not a pair, and cannot be one.** The table's last row reads *"`try` and `async`
blocks | a block expression"*, but Voltaic provides no executor (doc 03 §14.3, doc 01 §24) and
`async` **passes through** to Rust's own `async` block. The generated Rust *is* what the author
writes, so §1.3's own rule — *only the lowerings that generate more than the author wrote* —
excludes it. `try_block.sdx` covers the block-expression mechanism; `async` needed no pair and
has none.

### 10.3 The `try` pair failed first, for a reason that was not the lowering

Its first version had both halves call a `parse_it` helper. It failed, and the IR diff showed
why: rustc **inlined the helper into the hand-written half** and left a call to it through the
closure in the generated one. The two sides differed by an inlining decision about a *third*
function.

That is a false positive this harness can produce, and it is worth naming because the failure
looked exactly like a real one — a 200-line IR diff asserting the `try` lowering is not free,
when the lowering was fine. **A pair must leave the lowering as the only variable.** The
shipped version applies `?` to a parameter, so there is no helper to inline and nothing else
can differ.

### 10.4 What the register cell asked for and did not get

Row 44 says *"asserted within noise"*. §1.2 declined that before any code existed and the
decision held, so the register's cell is corrected by this row rather than satisfied by it.

**Watch latency is reported as not measured**, with the reason in `REPORT.md`: the figure
would be `notify`'s debounce plus the page cache, dominated by the filesystem and read as
though it described the compiler. §1.4's *losses reported* discipline is what made writing
that down the honest outcome rather than a gap. Transpile throughput *is* measured — 27 files,
5.7 MB/s on a named machine, in a disclosed debug build.

### 10.5 Two defects in this crate that only the blaze path showed

Both were found days later, by the per-crate build work, and both had been passing under
`cargo nextest` the whole time:

- `rustc_path()` read its baked `VOLTAIC_RUSTC` with `std::env::var`. `cargo::rustc-env` bakes a
  value in at **compile** time, reachable through `env!` and absent from the process
  environment, so the lookup always missed and left `PATH`'s NixOS `stub-ld` as the only
  candidate. Under `blaze test` all four tests failed; under cargo, `PATH` happened to hold a
  real rustc and hid it. The crate also gained the `build.rs` that bakes the value, which it
  had been relying on `voltaic-cli`'s copy of without being able to read it.
- Two clippy errors (`needless_borrows_for_generic_args`, `unnecessary_sort_by`) that the
  workspace-wide lint had never reached.

The lesson is AGENTS.md's, restated: a test that passes under one build command is not a test
that passes.
