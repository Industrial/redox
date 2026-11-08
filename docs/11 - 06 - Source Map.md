# Plan 06 — Source Map

Plan ID: `source-map`
Depends on: 04 (`emitter-determinism`), 05 (`diagnostics-miette`)
Milestone: 2 — *rustc's errors land on the user's source*
Status: `done`

---

## 1. What this plan is

The emitter learns to say **where each generated line came from**, and writes that down
beside the `.xrs` as a `.xrs.map` sidecar.

Nothing user-visible changes. This plan builds the one data structure plan 07 needs, and
nothing else: plan 07 parses `cargo check --message-format=json`, rewrites every span
through this map, and re-renders against the `.sdx`. Splitting the two is deliberate — a
map is testable on its own, against the emitter's own output, with no rustc in the loop.
If the map is wrong, plan 07 would report confident nonsense, so the map is worth proving
first and separately.

Doc 02 §6.5 calls this *"the largest single gap in the current spec"* and *"worth more to
Redox's usability than every item in Phase 5 of the spec's roadmap."* Both are fair. A
transpiler that reports

```
error[E0308]: mismatched types
  --> main.xrs:47:19
```

is pointing at a line the user never wrote, and no amount of polish elsewhere compensates.

### 1.1 Why there is no `serde`

The sidecar is written and read by hand, in a line-oriented text format. `serde` plus
`serde_json` would be two more dependencies and a derive macro to serialise four integers
per line — the same judgement that kept `thiserror` out in plan 05. The format is also
*readable*, which matters more than it sounds: a wrong map is diagnosed by opening the
sidecar next to the `.xrs`, and a page of JSON on one line does not support that.

---

## 2. Verified preconditions

Measured against the tree at plan 05, not assumed:

| Claim | How checked | Result |
|---|---|---|
| Every statement carries a span | `redox-ast/src/block/stmt.rs:15` | `pub span: Span` |
| Every item carries a span | `redox-ast/src/program/item/fn_decl.rs:13` | `pub span: Span` |
| Every block carries a span | `redox-ast/src/block.rs:10` | `pub span: Span` |
| The emitter writes into a plain `String` | `redox-emit/src/emit.rs` | `fn emit(program: &Program) -> String`, 12 helpers taking `&mut String` |
| `redox-diag` is the lowest crate | plan 05 | `Span` lives there; doc 06 §1.2 already names `source.rs` as the group *"[that] will group plan 06's span map"* |

### 2.1 The precondition that failed, and what it costs

Doc 02 §6.5's design item 1 is:

> **Line-preserving emission wherever feasible.** Make one `.sdx` line emit one `.xrs`
> line as the default discipline. This is achievable for most constructs and makes the map
> almost trivial.

It is **not** achievable, and the measurement is one command:

| Example | `.sdx` lines | `.xrs` lines |
|---|---|---|
| `hello.sdx` | 3 | 4 |
| `phase1.sdx` | 5 | 7 |
| `expressions.sdx` | 23 | 24 |
| `comments.sdx` | 8 | 9 |

Every generated block ends with a `}` on its own line, and Redox's source has no such
line — the off-side rule is the whole point. `phase1.sdx` has two functions and so gains
two lines.

The only way to preserve the count would be to put the brace on the last content line
(`a + b }`), and plan 04 forecloses that: it established as a *measured* property that
`rustfmt` leaves Redox's output unchanged, and `rustfmt` would move that brace. Trading a
measured guarantee for an untested one, to save four integers per line in a sidecar, is a
bad trade.

So doc 02 §6.5 item 1 is **corrected here**: line-preserving emission is not the
discipline, and the map is not trivial. It is, however, small — one record per generated
line — and the correction costs nothing but this paragraph. Items 2, 3 and 4 of that
section stand unchanged.

---

## 3. Concrete proof this plan targets

From doc 02 §6.5 item 4, adopted verbatim as the standard of proof:

> **Prove it like Emerald proved DWARF:** a test that compiles a `.sdx` with a *deliberate
> type error on a known line*, runs real rustc, and asserts the remapped diagnostic reports
> that exact `.sdx` line and column. Not an inspection — an executed test.

That test belongs to plan 07, because it needs rustc's diagnostic stream. Plan 06's
obligation is the half that can be proved without rustc, and it is the stricter half:

**For every example in `examples/`, every line of generated Rust either resolves to a
source position whose text the test can check, or is one of a small, enumerated set of
synthetic lines.** Not a sample — every line, with the exceptions listed rather than
tolerated.

A map that is 90% right is worse than no map, because the 10% is indistinguishable from
the 90% at the point of use. Totality is therefore the criterion, not accuracy on a
chosen example.

---

## 4. Leaves

### Leaf 1 — `leaf-origin-and-map`

`redox-diag/src/source/origin.rs` and `source/map.rs`.

```rust
pub struct Origin {
  pub span: Span,      // the source span of the construct that produced the line
  pub src_line: usize, // 1-based
  pub src_col: usize,  // 1-based, where the construct's text begins
  pub gen_col: usize,  // 1-based, where it begins on the generated line
}

pub struct SourceMap { /* one Option<Origin> per generated line */ }
```

`SourceMap::resolve(gen_line, gen_col) -> Option<(usize, usize)>` returns the source line
and column. The column arithmetic is a shift: `src_col + (gen_col - gen_col_of_origin)`,
floored at 1. That is exact when the generated line's content after its indentation is the
source's content verbatim — which holds for expression statements, the lines rustc
actually complains about — and approximate otherwise. §6 records where.

### Leaf 2 — `leaf-emitter-sink`

`redox-emit/src/emit/sink.rs`. A `Sink` owning the output `String` and the marks, with
`impl std::fmt::Write`, so the twelve `write!`-using helpers change only in the type they
take. `&mut String` becomes `&mut Sink` throughout.

`emit_with_map(program, source) -> (String, SourceMap)` becomes the real entry point;
`emit(program) -> String` stays, delegating and discarding the map, so plans 01–04's
callers and goldens are untouched.

### Leaf 3 — `leaf-marks`

`Sink::mark(span)` at each item and statement boundary. The sink attributes every line it
begins from a mark until the next one, so a construct spanning several generated lines
maps all of them, and a line with no mark before it — only the leading blank line between
items — maps to nothing.

### Leaf 4 — `leaf-sidecar`

`redox-diag/src/source/map/render_map.rs` and `parse_map.rs`. One header line, then one
record per generated line:

```text
redox-map 1 examples/phase1.sdx
1 1 1 4
2 2 3 5
3 -
```

`gen_line src_line src_col gen_col`, or `gen_line -` for a synthetic line. Round-tripped
by a test over every example. The CLI writes `<output>.map` beside the `.xrs`.

### Leaf 5 — `leaf-totality-proof`

`redox-cli/tests/source_map.rs`:

1. **Totality** — for every example, every generated line resolves or is synthetic, and
   the synthetic set is exactly `{closing brace, blank}`.
2. **Text agreement** — for every resolving line, the source text at the resolved position
   begins the same token the generated line's content does. This is the assertion that
   would catch an off-by-one, which a line count cannot.
3. **Round trip** — serialise, parse, compare.
4. **A known line, by hand** — `phase1.sdx`'s `a + b` is source line 2; the generated
   `    a + b` is line 2 of the `.xrs`; the map says so, and the test names both numbers
   literally so a reader can check the claim without running anything.

### Leaf 6 — `leaf-tests-and-gate`

Full gate, doc 04 status, doc 06 §10 rows for the new checks.

---

## 5. Acceptance criteria

| # | Criterion |
|---|---|
| 1 | Every generated line either resolves through the map or is one of an enumerated set of synthetic lines |
| 2 | The resolved position's source text agrees with the generated line's content |
| 3 | `phase1.sdx`'s `a + b` maps from `.xrs` line 2 to `.sdx` line 2, asserted with both numbers written out |
| 4 | The sidecar round-trips for every example |
| 5 | `redox build` writes `<output>.map` beside the `.xrs` |
| 6 | `emit(program)` still exists and still returns exactly what it did, so plans 01–04's goldens are byte-identical |
| 7 | No new third-party dependency |
| 8 | Full gate green |

---

## 6. Out of scope, and what it discloses

| Deferred | To | Note |
|---|---|---|
| Remapping rustc's diagnostics | **plan 07** | This plan builds the map; plan 07 is the first consumer |
| Column exactness on synthesised lines | — | `fn main() {` has no source column; the map resolves it to the declaration's own column, which is the honest answer rather than a fabricated one |
| Multi-line statements | **plan 09** (`continuations`) | A statement spanning two source lines via `\|>` maps to its *first* source line. Correct but coarse; it becomes visible when continuations carry real code |
| A binary map format | — | Not until a map is large enough to measure, which it is not |

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format
```

---

## 8. Emerald reuse

**Consulted, not lifted:** Emerald plan 35 (`debug-info`), which emits DWARF line tables
through inkwell's `DebugInfoBuilder`. The mechanism does not transfer — Redox's output is
text, not machine code — but two things do: the *standard of proof* (a scripted session,
not an inspection), and the structural lesson that it was plan **35**. Emerald shipped
thirty-four plans before a user could see a `.em` line number in a debugger. Redox does it
at plan 06, before there is enough language for the gap to become normal.

**Not applicable:** Emerald has no source-map analogue, because it has no generated
intermediate source. This is the one place where being a transpiler makes the problem
*harder* rather than easier, and where Emerald has nothing to offer.

---

## 9. Decision log

- **One record per generated line, not per span.** A span-level map would be more precise
  and much larger, and rustc reports positions by line and column — so a line-indexed map
  is what the consumer actually queries. Precision beyond the consumer's resolution is
  cost without benefit.

- **Doc 02 §6.5's line-preserving discipline is corrected rather than attempted** (§2.1).
  Braces need their own lines, and plan 04's rustfmt guarantee is worth more than the
  simplification.

- **The map lives in `redox-diag`, not `redox-emit`.** The emitter produces it and the
  driver consumes it; putting it in the producer would make the consumer depend on the
  emitter for a type that is not about emitting. Doc 06 §1.2 anticipated this.

- **`emit` keeps its signature.** `emit_with_map` is added beside it. Plan 04's goldens are
  a measured property of `emit`'s output; changing its signature to thread a map through
  would have put that property at risk for no gain.

- **The sidecar is text, hand-written and hand-parsed** (§1.1). Readable, diffable, and no
  dependency.

- **Synthetic lines are enumerated, not tolerated.** The totality test asserts the
  synthetic set is exactly `{closing brace, blank}`. If a later plan emits a third kind of
  synthetic line, that test fails and the author has to say so out loud.

---

## 10. Result

`done`. 191 tests pass; `fmt`, `clippy -D warnings` and the full gate are green.

| # | Criterion | Evidence |
|---|---|---|
| 1 | Every generated line resolves or is enumerated-synthetic | `source_map.rs`: `every_generated_line_is_accounted_for`, over all four parseable examples |
| 2 | The resolved position agrees with the source text | `a_resolved_position_agrees_with_the_source_text` — checks the span's text appears on the resolved line, which a line count cannot |
| 3 | `phase1.sdx` line 2, written out | `phase1_line_two_maps_to_source_line_two`: `map.resolve(2, 5) == Some((2, 3))` |
| 4 | The sidecar round-trips | `the_sidecar_round_trips`, plus eight unit tests in `parse_map.rs` for the ways a sidecar can be malformed |
| 5 | `redox build` writes the sidecar | `build_writes_a_parseable_sidecar` reads what the binary actually wrote |
| 6 | `emit` unchanged | `emit_still_returns_what_emit_with_map_returns`, and plan 04's goldens are byte-identical |
| 7 | No new dependency | `Cargo.lock` unchanged |
| 8 | Full gate green | `moon run :check :lint :test :cargo-check :cargo-test` |

A map, in full, for `comments.sdx` — the case worth reading because it has trivia at both
ends and two synthetic lines in the middle:

```text
redox-map 1 examples/comments.sdx     // `.xrs` line        → `.sdx` line
1 1 1 1                               // comment            → 1
2 2 1 1                               // comment            → 2
3 -                                   // blank              → synthetic
4 4 1 1                               // fn main() {        → 4
5 5 3 5                               // println!("one")    → 5
6 6 3 5                               // println!("two")    → 6
7 -                                   // }                  → synthetic
8 -                                   // blank              → synthetic
9 8 1 1                               // trailing comment   → 8
```

### 10.1 What the totality test caught

One defect, and it was a design error rather than a slip: **comment lines were being
recorded as synthetic.** The first implementation called `unmark()` before emitting leading
trivia, on the reasoning that a comment belongs to no construct. The test rejected
`hello:1` — `// The smallest Redox program.` — as "neither a closing brace nor blank", and
it was right to. A comment is something the author wrote, at a position they can be shown;
rustc can complain about a doc comment, and plan 07 would have had nowhere to point. Trivia
now marks its own span, and the three comment lines in `comments.sdx` map to their true
source lines.

The general shape is worth naming, because it will recur: **the easy thing to do with a
line you are unsure about is call it synthetic, and that is exactly the wrong default.** A
synthetic line is a promise that no source position exists. Asserting the synthetic set
*exactly*, rather than bounding its size, is what turns that promise into something a test
can refuse.

### 10.2 Two architecture violations, caught by the architecture tests

Neither was visible while writing the code; both were named precisely by
`one_construct_per_file.rs`:

- **`render_map.rs` declared two constructs** — the `VERSION` constant alongside the
  function. `VERSION` moved to `source/map/version.rs`.
- **`emit.rs` declared two** — `emit` and `emit_with_map`. The second moved to
  `emit/with_map.rs`, where doc 06 §2.3's parent-plus-stem rule makes `emit_with_map` the
  *correct* name for the file: parent `emit` plus stem `with_map`. The rule did not merely
  permit the new home, it picked it.

And one naming violation that improved the design: `SpanMap` in `source/map.rs` failed
§2.3, because the parent is `source`, so the only legal names are `map` and `source_map`.
Renamed to **`SourceMap`** — which is both what the rule requires and what the thing is
usually called. Three plans running, these checks have found things a reading would not.

### 10.3 Disclosures

- **A multi-line statement maps to its first source line.** The `Sink` attributes by mark,
  and a statement's mark is its whole span, so a statement continued across two `.sdx`
  lines with `|>` resolves to the line it began on. Correct but coarse; it matters once
  continuations carry real code (plan 09).

- **Column mapping is exact only where the content is copied verbatim.** `println!("{}",
  add(10, 20))` bears no character-by-character relation to `print add(10, 20)`, so a
  column inside it shifts by the indentation difference and no more. For `a + b` — and for
  every expression emitted as itself — it is exact. Plan 07 will show how much this
  matters in practice; the honest statement for now is that the *line* is reliable and the
  *column* is reliable only on lines the emitter did not restructure.

- **The sidecar stores no span.** A parsed `Origin` carries `(0, 0)`. The four integers a
  consumer needs are the positions; a byte range would be a fifth field able to disagree
  with the other four. The round-trip test compares what the format claims to preserve,
  and says so.

- **`emit("")` produces a map of meaningless positions.** `emit` passes an empty source, so
  every origin resolves against nothing. It discards the map, so nothing can observe this —
  but a future caller reaching for `emit` and then wanting a map would get a silently wrong
  one. The reason `emit_with_map` takes the source explicitly rather than defaulting is to
  make that mistake require writing `""`.
