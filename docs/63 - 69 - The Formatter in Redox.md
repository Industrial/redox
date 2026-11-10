# Plan 69 — The Formatter in Redox

Plan ID: `fmt-in-redox`
Depends on: 68 (`ast-in-redox`), 64 (`macro-invocation`), 34 (`formatter`)
Milestone: 10 — *self-hosting*
Status: `planned`

---

## 1. What this plan is

**`crates/redox-fmt` is written in Redox**, by plan 68's harness and on plan
68's terms: 57 `.sdx` files, the generated Rust committed in `src/`, the
workspace compiled against it.

And it is the row that **decides whether the other ten happen**. Plan 68's
value was not self-hosting; it was nine surface gaps and three comment-loss
defects, one of which no other test in the gate could have caught. This row is
the second data point, and its §10 has to report the yield whether or not it
flatters the effort.

### 1.1 The measurement that chose this crate, and corrects a published figure

`docs/CONFORMANCE.md` reads **12 blocking occurrences, all `as` casts**, and
the figure is honest about what it counts: the five constructs
`docs/GRAMMAR.md` grades `UNREACHABLE`. It does not know about the nine gaps
plan 68 found by porting a crate, and counted across the eleven crates still
in Rust those are **six times larger**:

| | hits | crates |
|---|---:|---:|
| **graded `UNREACHABLE`** — what `CONFORMANCE.md` counts | **29** | 8 |
| `matches!` | 61 | 7 |
| inline `#[cfg(test)] mod tests` | 61 | 7 |
| `for` over a pattern | 19 | 7 |
| open-ended range | 16 | 7 |
| `Self::` as a path expression | 7 | 4 |
| a method passed as a value | 4 | 4 |
| a declaration-bodied macro | 3 | 1 |
| `vec![elem; n]` | 2 | 1 |
| **found by plan 68, counted by nothing** | **173** | |
| **total** | **202** | |

467 files and 28,316 code lines, so 202 sites is 0.7% of lines — close by that
measure and not by the one that matters, which is how many **rows** stand in
the way. §6 sorts them.

**`redox-fmt` has six of the 202.** Three `matches!`, one `for` over a
pattern, one open-ended range, one method passed as a value. No `as` casts, no
inline test modules, no declaration-bodied macros. It is the cleanest crate in
the workspace after the one already ported, which is why it is next.

### 1.2 Its shape is the opposite of `redox-ast`'s, and that is the point

A second port of the same shape would measure nothing. These two could not be
less alike:

| | `redox-ast` | `redox-fmt` |
|---|---|---|
| files | 53 | 57 |
| code lines | 1,411 | 2,910 |
| data declarations | 38 files of them | **none** |
| `impl` blocks | 6 | **none** |
| free functions | 3 | **57 — one per file** |
| `format!` calls | 2 | **125** |
| `.join(` | 1 | 24 |
| closure arguments | 9 | 27 |

`redox-ast` is *"syntax tree types, no logic"* and the port was mostly
declarations; plan 68's translator did 38 of its files mechanically and is
**useless here**, because it translates declarations and this crate has none.
Every file is one function and every one is hand-written.

So the surface under test is different: the **macro call** plan 64 shipped
three rows ago, exercised 125 times; string building; iterator chains;
closures; `if let` and `let … else`. Plan 68 exercised almost none of that.

### 1.3 The hazard that is not one, checked first

A formatter prints Redox, so its own string literals carry Redox syntax as
**data** — and `#{` in a Redox string opens an interpolation. If `redox-fmt`
held `"#{"` as a literal, the port would read the formatter's own output
spelling as a hole.

Counted before this plan was written: **all eight `#{` occurrences are in doc
comments**, where trivia is verbatim and nothing interpolates. One string
literal carries Redox syntax — `macro_call_text.rs`'s `" \\->\n"` — and `\\`
is an ordinary escape in both languages.

The brace case is already solved and already exercised: a `{` in a Redox
string is literal and the emitter doubles it, which is how plan 68's
`pattern.sdx` emits `format!("{} {{ {} }}", …)`. Confirmed there rather than
assumed here.

---

## 2. Verified preconditions

Measured 2026-11-10, against `redox` at `32dea7a` — the commit that landed
plan 68.

**The harness exists and works.** `crates/redox-cli/tests/ast_port.rs` is the
mirror assertion, the regeneration test with its staging guard, the
escape-hatch check and leaf 4's doc-comment count. Leaf 0 of this row
generalises it to two crates rather than writing a second copy.

**`redox-fmt` is 57 of 57 files blocker-free** by the five graded constructs,
and carries six of the 202 sites §1.1 counts.

**Plan 68's nine gaps and their workarounds are known**, which is the
difference between this row and that one: an open-ended range binds its end
first, a `for` over a pattern derefs in the body, `matches!` becomes a `case`
ladder, a method passed as a value becomes a lambda, `Self::name` becomes the
type's own name. None has to be rediscovered.

**Two clippy allows will be needed**, and plan 68 §10.3 is why:
`match_like_matches_macro` because `matches!` is unreachable, and `no_effect`
if any `()`-returning function needs plan 55 §1.6's written unit tail. Both
are statements of fact, not suppressions, and the second has a successor owed.

**`format!` is reachable and exercised.** Plan 68's port used it, and
`format! \-> "#{a}-#{b}"` emits `format!("{}-{}", a, b)`. Interpolation
splices, so a 125-call crate needs nothing new.

---

## 3. Concrete proof this plan targets

**`cargo check --workspace` compiles `redox-fmt` from Rust generated by Redox,
and every test passes** — including `redox-fmt`'s own 1,463 lines of tests,
which assert the formatter's output byte for byte. A printer that prints one
character differently fails its own suite, so this is a sharper oracle than
`redox-ast` had.

Four more:

1. **`redox fmt --check` passes on the whole `.sdx` corpus**, formatted by a
   formatter compiled from Redox. The row's one genuinely circular assertion
   and the one worth having: the formatter formats its own source.
2. **125 `format!` calls survive**, which is plan 64's macro invocation
   exercised at a scale no example reaches.
3. **Every doc comment survives as a doc comment**, by the count leaf 4
   already applies — now over a crate with no declarations to hang them on,
   so they sit on functions instead.
4. **The yield is reported**, whether it is nine findings or none. §10.

---

## 4. Leaves

### Leaf 0 — `leaf-two-crate-harness`

`ast_port.rs` becomes a harness over a **list of ported crates** rather than
one hard-coded path, and `conformance/redox-fmt/` joins
`conformance/redox-ast/`. The staging guard checks each crate it is given.

One test file, not two: the four assertions are the same four, and a second
copy would be the one-predicate-two-positions defect plan 34 §10.31 found five
times and plan 61 found again.

### Leaf 1 — `leaf-port-the-printers`

The 57 files. Every one is a single function, so the leaf is bounded by count
rather than by shape, and the order is smallest first — `indent_text` is three
lines, `expr_text` is 305.

### Leaf 2 — `leaf-the-six-gaps`

The six sites §1.1 counts, each with the workaround plan 68 established, and
each **named in the `.sdx`** so a reader meets the reason where the oddity is
rather than in a plan document.

### Leaf 3 — `leaf-docs-and-register`

`docs/CONFORMANCE.md`'s worked-port section names two crates;
`docs/GRAMMAR.md` keeps its grades; milestone 10's table carries the **202**
rather than the 29, because the 29 is what made the distance look shorter than
it is. `docs/STATUS.md` regenerates.

---

## 5. Acceptance criteria

1. `crates/redox-fmt/src/**/*.rs` is generated from
   `conformance/redox-fmt/**/*.sdx`, one file for one file.
2. `cargo check --workspace --all-targets` passes with no change to any other
   crate.
3. Every test passes, unchanged — **including `redox-fmt`'s own**, which
   assert the formatter's output byte for byte.
4. `redox fmt --check` passes on the whole `.sdx` corpus, run by the
   Redox-compiled formatter.
5. Regenerating reproduces what is committed, byte for byte, and the staging
   guard refuses a generation that does not compile.
6. Every `-- |` and `-- !` reaches the generated Rust as `///` and `//!`.
7. No escape hatch anywhere in the source.
8. One harness for both crates, not two.
9. `docs/CONFORMANCE.md` and `docs/STATUS.md` regenerated; milestone 10's
   table carries the 202.
10. **§10 reports the yield** — the defects and gaps this port found, or the
    fact that it found none. The row is a measurement and the measurement is
    the deliverable.

---

## 6. Out of scope, and what it discloses

**The other ten crates are not decided by this row**, they are *informed* by
it. §10's yield is the input to that decision and §1 is the argument for
taking it seriously in both directions.

**The 202 sort into four kinds, and only one is porting.** Written here
because the next reader of `docs/CONFORMANCE.md` will otherwise price
self-hosting at 12 occurrences:

- **Reachable today, awkwardly — about 109 sites.** All 61 inline test
  modules, because item-level `test "…"` already emits
  `#[cfg(test)] mod __rdx_tests { use super::*; }` and sees the file's private
  items; plus the ranges, `for` patterns, `Self::` paths, `vec![x; n]` and
  method-as-value workarounds. Reachable is not the same as fine: this is 109
  deliberate uglifications.
- **Three parser rows the language wants anyway** — `for` over a pattern,
  open-ended ranges, `Self::` as a path expression. Each is small, and the
  range one closes a **live defect**: `r = 2..` silently swallows the
  statement below it, which is a fifth deferred case found after milestone 9
  declared that class empty.
- **Two surface decisions, the expensive kind.** A macro form that can take a
  **pattern**, for `matches!`'s 61 sites — otherwise it is `case` ladders and
  a `match_like_matches_macro` allow in seven crates. And `as`, 29 sites,
  where the nearest working thing is `try_from` and that is a *different
  program*, since `as` truncates.
- **One genuinely hard case.** `thread_local!` — a macro whose body is a
  **declaration**. Plan 64 §6 disclosed that `name! \-> e` cannot carry it. It
  is in exactly three files, all in `redox-emit`, all scope guards from plans
  59 and 60. The escape hatch is the answer and criterion 7 bans it in a port,
  so that crate needs the macro form or a stated exemption.

**100% cannot mean deleting the Rust.** Doc 02 §4.8 says the Rust
implementation stays *permanently*, correcting doc 01 §34. The coherent target
is what plan 68 did once: every crate has a `.sdx` source and the committed
`.rs` is generated. Both representations, with a regeneration test — and that
is a cost, paid forever, which §10's yield has to be weighed against.

**`redox-fmt`'s tests are not ported.** 1,463 lines of them, and they are what
makes criterion 3 sharp — a Rust test asserting the output of a
Redox-generated printer is a better oracle than a ported test asserting it.
Plan 68 left `redox-ast`'s tests alone for the same reason.

---

## 7. Gate

```bash
moon run :check          # the whole workspace, against both generated crates
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # every test, including redox-fmt's own
moon run :format         # treefmt
```

Plus: the regeneration test for both crates; the doc-comment count; no `rust`
hatch; the mirror assertion; `redox fmt --check` over the corpus;
`docs/CONFORMANCE.md` and `docs/STATUS.md` re-blessed and the diffs reviewed.

---

## 8. Emerald reuse

None, and for the reason plan 68 gives: Emerald never attempted this. What
transfers is **plan 68's harness and its nine gaps**, which is the first time
in this project a self-hosting row has had a predecessor to inherit from
rather than a measurement to make.

---

## 9. Decision log

1. **`redox-fmt`, because its shape is the opposite of `redox-ast`'s.** §1.2.
   A second port of the same shape would measure nothing; this one exercises
   the macro call 125 times where the first exercised it twice.
2. **One harness for two crates.** A second copy of four assertions is the
   defect plan 34 §10.31 found five times.
3. **The yield is a deliverable**, criterion 10. A row whose purpose is to
   inform a decision has to report what it found even if that is nothing,
   because "nothing" is the answer that would stop the effort.
4. **The 202 goes in the register.** `docs/CONFORMANCE.md`'s 12 is honest
   about its own scope and misleading about the distance, and a reader pricing
   self-hosting from it would be out by a factor of six.
5. **The tests stay in Rust.** A Rust test asserting a Redox-generated
   printer's output byte for byte is a sharper oracle than a ported one.

---

## 10. Result

*Not yet executed.*
