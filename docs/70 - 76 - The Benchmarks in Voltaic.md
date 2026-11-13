# Plan 76 — The Benchmarks in Voltaic

`voltaic-bench`'s 13 files, 597 lines. Row 72's fourth successor, and the first
one whose output is a **document** — `benchmarks/REPORT.md` — so the port had
to preserve prose byte-for-byte as well as behaviour.

---

## 1 What it is

| | |
|---|---|
| Source | `crates/voltaic-bench/sdx/**` — 13 files, 353 lines of Rust expressed |
| Generated | `crates/voltaic-bench/src/**`, untracked from this commit |
| Tests | 4, unchanged |
| `as` casts | **1 → 0** — this crate's one entry in `docs/CONFORMANCE.md` |
| Seed | **not refreshed** — byte-identical, so no version skew |

It is also the first **binary** crate ported, so its `main.rs` is generated and
a clean clone needs `./bootstrap.sh` before `cargo run -p voltaic-bench` works.

---

## 2 Prose had to survive, and that shaped the port

`render` builds the report from three long literals written with Rust's
`\`-at-end-of-line continuations, so each paragraph is **one line** of runtime
text wrapped across many lines of source. Voltaic has no such continuation: a
string spans lines only as a `"""` block, and that would put a real newline at
every wrap and hard-wrap the report.

So the three are **extracted from `main.rs` and un-escaped by the batch
script**, not retyped — the same treatment `keyword.rs`'s two tables got in
plan 74, for the same reason: 2,900 characters of prose typed out is 2,900
chances to change a word, and nothing in the gate compares `REPORT.md`'s text.

**Verified by running it.** The regenerated report differs from the committed
one only from line 29 — the measured numbers — because the committed report
predates nine new examples. Every line of prose above it is identical. That is
the check the harness cannot make, and it took one command.

---

## 3 The `as` cast, removed

`total_time.as_secs_f64() / total_bytes as f64` is this crate's one blocker in
`docs/CONFORMANCE.md`, and there is no `From<usize> for f64` to replace it
with. `u32::try_from` and `f64::from` express it without a cast, and the
`unwrap_or 0.0` cannot mislead: `per_byte` is then zero and the
`if per_byte > 0.0` guard the Rust **already has** prints `0.0 MB/s`. So a
corpus somehow larger than 4 GB reports nothing rather than something wrong —
which is the right failure for a measurement.

---

## 4 What it found

### 4.1 Module docs must stay first, and the writer hoisted imports above them

`main.sdx` is the first ported file whose head carries **both** `-- !` module
docs and imports. The body writer put the merged imports at the top
unconditionally, so `//!` landed after the `use` lines and rustfmt refused the
output with `E0753: expected outer doc comment`.

Three crates went by without it because their `lib.sdx` heads were never
rewritten by `write()` — the docs were already above the imports and stayed
there. Every remaining crate's `lib.rs` or `main.rs` would have hit it.

### 4.2 `rust` is a keyword, so a field cannot be called `rust`

`Pair`'s `rust: PathBuf` — plan 64 gave `rust` the layout-delimited escape
hatch, so the name is reserved. `r#rust` is the spelling, and it is transparent
to the one thing that reads the field: `tests/zero_cost.rs` is hand-written
Rust and stays Rust, where `r#rust` and `rust` are the same identifier.

**Two keyword collisions in two crates** — `data` in plan 75, `rust` here — on
names the Rust chose freely. The reserved list is long (`type`, `module`,
`test`, `case`, `of`, `is`, `in`, `or`, `not`, `do`, `with`, `where`, `try`,
`catch`, `guard`, `pure`, `errors`, `class`, `instance`, `implements`), so the
remaining six crates will meet more, and **the translator could prefix `r#`
itself** rather than leaving a parse error for a human to diagnose. Owed to a
successor row.

### 4.3 `.char_indices` is the answer to the annotated-closure gap

`rest.find(|c: char| …)` needs its annotation for inference, and a Voltaic lambda
parameter is a bare name — a gap recorded in plans 73, 74 and 75 and worked
around three different ways.

`.char_indices` is not a fourth workaround, it is the answer: it yields
`(usize, char)`, so `p.1` is a `char` and there is nothing to infer, and `p.0`
is the **byte** index `str::find` returns. Identical semantics, no annotation.
Worth keeping for every remaining crate.

### 4.4 `while_let_loop` is the third lint about an absent pattern

`while let Some(at) = rest.find('%')` has no Voltaic spelling, so `alpha_renamed`
is a `loop` with a `case` whose `None` arm breaks — and clippy asks for the
form that does not exist. That is now three lints saying one thing:
`match_like_matches_macro`, `single_match`, `while_let_loop`. All three are
*Voltaic cannot pass a pattern*.

### 4.5 A batch that patches another batch lands one build late

`p76b4` fixes a lambda in `p76b3`, and batches run in order — so the fix
applied *after* `p76b3` had already written its file, and the build had to run
twice. Plan 73 recorded the same thing for `p73b1fix`; recording it again is
the point at which it belongs in `port.sh`'s header rather than a plan.

---

## 5 Criteria

| | Criterion | Met |
|---|---|---|
| 1 | All 13 files are Voltaic | yes |
| 2 | `voltaic-bench` is in `seed/PORTED`, and `port.rs`'s five assertions cover it | yes |
| 3 | `crates/voltaic-bench/src/` is gitignored and out of the index | yes |
| 4 | `crates/voltaic-bench/sdx/` is in the index | yes |
| 5 | Every test survives | yes — 4 before, 4 after |
| 6 | `REPORT.md`'s prose is byte-identical | yes — verified by running it |
| 7 | Zero `as` casts | yes — 1 → 0 |
| 8 | `moon run :check :lint :test :sdx-format-check` green | yes |
| 9 | A clean clone bootstraps it | yes |
| 10 | `stage1 == stage2` still holds | yes |

---

## 6 Owed to successor rows

- **The translator should prefix `r#` on a keyword-named field or parameter**
  (§4.2), rather than leaving a parse error.
- **Six crates now carry the `no_effect` allow.** Plan 75 §2.4 called
  repeating the paragraph the wrong answer; it has happened once more. The
  decision is owed either a workspace-level allow or a change to plan 55 §1.6.
- Everything plan 74 §9 lists, unchanged.

---

## 7 Next

Six crates, roughly 20,300 lines: `voltaic-mcp` (808), `voltaic-lsp` (934),
`voltaic-cli` (1,376), `voltaic-driver` (2,981), `voltaic-emit` (6,203),
`voltaic-parser` (8,223).
