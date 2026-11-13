# Plan 83 — The Seed in Voltaic

`voltaic-seed`'s 6 files, 298 lines. **The thirteenth crate**, and the one that
makes the sentence true: no crate's `src/` is hand-written Rust any more.

Plan 82 claimed the conversion was finished and it was not. `voltaic-seed` was
left out of twelve rows of porting, and the claim survived because
`docs/CONFORMANCE.md`'s own generated prose said *"The one crates that are not
ported"* — ungrammatical, and read past.

The crate is the bootstrap tool, so the first question was whether porting it
is even sound. It is, and the reason is one line of `bootstrap.sh`: the clean
clone runs **`wasmtime run --dir . seed/voltaic1.wasm`**, the committed blob, and
never builds `voltaic-seed` from source. The blob regenerates all thirteen `src/`
trees before `cargo build` is reached, so the seed's own source is a thing the
seed produces, not a thing the bootstrap needs.

And the result is stronger than "it compiles": rebuilding the blob from the
**generated** Rust produced a wasm **byte-identical** to the one plan 82
committed from the hand-written Rust. `seed/SEED.md` needed no change.

---

## 1 What it is

| | |
|---|---|
| Source | `crates/voltaic-seed/sdx/**` — 6 files, 313 lines |
| Generated | `crates/voltaic-seed/src/**`, 302 lines, untracked from this commit |
| Tests | 2, unchanged — same names |
| List | `seed/PORTED` gains `voltaic-seed 83`, last: nothing depends on it |
| Seed | rebuilt and **byte-identical**, 928,389 bytes |
| Doc comments | 116 in, 116 out — exact |

**Thirteen crates, 532 files.** `docs/CONFORMANCE.md`'s remaining 2 blocking
occurrences are both in `voltaic-lsp`, and both are `as` casts in files no row
has reached.

---

## 2 What it found

### 2.1 Porting the seed is sound, and `bootstrap.sh` is why

The circularity a reader expects is not there. Written out:

| | needs | provides |
|---|---|---|
| `seed/voltaic1.wasm` | nothing — it is committed | all thirteen `src/` trees |
| `cargo build` | every `src/` | the native `voltaic-seed` |
| `moon run :seed-refresh` | the native `voltaic-seed` | the next blob |

The only edge into `voltaic-seed/src` comes from `cargo build`, which runs
*after* the blob has written it. That is the same relationship `voltaic-driver`
has had since plan 80 — the seed is built from `voltaic-driver`, whose `src/` the
seed generates — so this row adds no new cycle, it closes the one that was
already there.

`stage1_equals_stage2` is the assertion that it closed correctly, and it
passes: the blob emits the sources the native binary was built from, and that
binary emits the same sources again.

### 2.2 A comment in two positions parses and is then silently deleted

New, and the quieter half of a defect plan 69 §10.5 already recorded.

```text
answer : i32
-- A comment between the signature line and the equation.
answer = 1

shape n =
  case n of
    0 -> 0
    -- A comment between two case arms.
    _ -> 1
```

Both parse. Both comments are then **absent from the generated Rust**, and
absent from `voltaic fmt`'s output as well:

```rust
fn answer() -> i32 {
    1
}

fn shape(n: i32) -> i32 {
    match n {
        0 => 0,
        _ => 1,
    }
}
```

So it is not a formatter defect. The **parser** has no attachment point for
either position, and the emitter and the formatter lose the comment together —
which is why nothing in the gate notices. `AGENTS.md`'s rule is *"Comments are
trivia, never discarded. They must reach the generated `.xrs`"*, and doc 02
§10.1 is what corrects doc 01 §4.1 on it.

**It is louder in plan 69's form and worse in this one.** §10.5's comment
between two pipeline stages fails to parse — `RDX0102`, a fixture, a diagnostic
a reader cannot miss. These two produce a program that compiles and is missing
a sentence the author wrote.

Three positions do survive, measured rather than assumed:

| Position | Survives |
|---|---|
| Above the signature line, under its `-- \|` block | yes |
| Above the `case` | yes |
| First line inside an arm's body block | yes |

Both of this crate's comments moved — the `main` note above its signature line,
`run`'s *"Collected, not returned"* into the `Err` arm's body, where it still
sits beside what it explains. That is what plan 69 did with its two sites and
for the same reason: a port row is not where trivia attachment gets rebuilt.
Fixing it is a field on an AST node, a parser change, and a printer change in
both `voltaic-emit` and `voltaic-fmt` — three crates and a bless cascade, which is
a row.

**And the check that would have caught it does not exist yet.**
`every_doc_comment_reaches_the_generated_rust` counts `-- |` against `///`;
these are ordinary `--` comments, so it is silent by construction. The owed
fidelity check — asked by rows 79, 80, 81 and 82 — should count `--` too.

### 2.3 The sdx formatter never saw the new files, because the corpus is `git ls-files`

`port.sh` runs `./sdx-fmt.sh` once the TODO count reaches zero, and
`sdx-fmt.sh` defines its corpus as `git ls-files '*.sdx'` minus an exclusion
list. A crate being ported for the first time has no tracked `.sdx` at that
moment, so **its files are not in the corpus and are not formatted** — and
`:sdx-format-check` then fails after they are staged.

Every earlier row staged its `sdx/` before the first `port.sh` run by accident
of ordering. This one did not, which is how the gap showed. It is also how the
defect in §2.2 was found: the formatter's first pass over these files is what
deleted the two comments.

No fix here. `git ls-files` is the right definition of *"every `.sdx` that is
meant to be canonical"* — the alternative is formatting scratch files — and the
ordering is a one-line note for the next port rather than a tool change:
**stage `sdx/` before the run whose formatter output you intend to keep.**

### 2.4 Two generated sentences went wrong at exactly the milestone they record

`docs/CONFORMANCE.md` is generated by `conformance.rs`, and its prose had no
zero case:

- `word()` held `"No"` through `"Twelve"` and fell back to digits, so the
  heading read *"## 13 crates are no longer a count"* among twelve siblings
  spelled as words;
- the closing paragraph read *"The **no** crates that are not ported are
  hand-written Rust and stay that way"*.

Both are fixed in the generator rather than in the document, which is the
whole point of a generated document: `word()` reaches `"Fourteen"`, and
`remainder()` answers the zero case with what plan 72 §6 actually names — the
integration tests and `crates/voltaic-driver/build.rs`. The sentence *"the
compiler is now **partly** compiled from Voltaic"* loses its qualifier the same
way.

This is the second time this row's milestone was obscured by its own
bookkeeping. The first is in the header above: plan 82 read *"The one crates
that are not ported"* and did not stop at it.

---

## 3 Criteria

| | Criterion | Met |
|---|---|---|
| 1 | All 6 files are Voltaic | yes |
| 2 | `voltaic-seed` is in `seed/PORTED` and `port.rs` covers it | yes |
| 3 | `crates/voltaic-seed/src/` is gitignored and out of the index | yes |
| 4 | `crates/voltaic-seed/sdx/` **is** in the index | yes |
| 5 | Every test survives | yes — 2 before, 2 after, same names |
| 6 | `moon run :check :lint :test :sdx-format-check` green | yes |
| 7 | No escape hatch | yes |
| 8 | A clean clone bootstraps it | yes — thirteen trees now, including this one |
| 9 | `stage1 == stage2` still holds | yes |
| 10 | No doc comment lost | yes — 116 → 116, exact |
| 11 | The rebuilt seed still works | yes, and it is **byte-identical** |
| 12 | `voltaic fmt` round-trips every ported `.sdx` | yes, after §2.2's two comments moved |
| 13 | No crate's `src/` is hand-written Rust | **yes** |

---

## 4 Next

**The conversion is finished.** Every one of the thirteen crates' `src/` is
generated from Voltaic. What remains Rust is what plan 72 §6 named and this row
did not touch:

- **18,790 lines of integration tests.** `AGENTS.md` requires public-API tests
  to be integration tests, and plan 69 decision 5 is that a Rust test asserting
  over Voltaic-generated code is the right shape. Never ported, never untracked.
- **`crates/voltaic-driver/build.rs`.** Cargo runs it on the host before anything
  exists, and it is what bakes in a verified `rustc` path.

Plan 72 §6 left one question open — *"Leaf 7 must say which reading the project
is taking"* of doc 02 §4.8's *"keep the Rust implementation permanently"*. This
row answers it by exhaustion: the reading taken is the tests and the build
script, not a parallel hand-written compiler. There is no second implementation
and no row proposing one.

Owed to a successor, from this row:

- **A comment between a signature line and its equation, and between two
  `case` arms, is silently discarded** (§2.2). Parser attachment, then both
  printers. Third sighting of the class after plan 69 §10.5, and the first that
  is silent.
- **The doc-comment fidelity check should count `--` as well as `-- |`**, and
  should be `port.rs`'s seventh assertion. Asked by rows 79, 80, 81 and 82;
  §2.2 is the first defect it would have caught and did not.
- **Doc 03 §1.1 should say that `.name` is a method in prefix position and a
  field in postfix position** (row 82 §2.1).
- **`Pattern::rust_text` should parenthesise an `@` over an alternation**
  (rows 81 and 82), which is `E0408` today.
- **`line_cap.rs`'s cap meets generated Rust, ~5% larger than the Rust it
  replaces** (row 82 §2.2).
- **A macro's format-template argument should be known by name** (rows 80, 81).
- **A keyword collision needs its own diagnostic** (rows 75 onwards).
