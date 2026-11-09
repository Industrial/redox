# Plan 48 — Conformance and Self-Hosting

Plan ID: `conformance-self-hosting`
Depends on: 45 (`reachability-residue`), 46 (`grammar-inventories`)
Milestone: 7 — *tooling*
Status: `planned`

---

## 1. What this plan is

Doc 02 §4.8 reframed self-hosting and wrote the acceptance criterion this row has to answer:

> Phrase the acceptance criterion as **"the compiler's source can be expressed in Redox
> without the inline-Rust escape hatch,"** which is the property actually worth proving.

It also predicted how the answer would be found — *"it will find every `UNREACHABLE` Rust
construct in §4.1's inventory the hard way"*.

**Plan 46 found them the easy way.** So this row does not port 24,159 lines to discover what is
already written down. It **counts** how often each known blocker appears in the compiler's own
Rust, which answers the criterion for every file instead of for a sample, and then ports one
real module to prove the counting method describes something real.

The answer is **no**, and §1.2 says what it would take.

### 1.1 Verified preconditions

Measured 2026-11-10 across all **430** Rust source files in `crates/*/src`, with comments
dropped and string bodies blanked.

| Blocker (plan 46 grades each `UNREACHABLE`) | Occurrences |
|---|---:|
| **Macro invocation** | **848** |
| `as` cast | 9 |
| `unsafe trait` | 0 |
| `unsafe impl` | 0 |
| `extern "C" fn` | 0 |
| Macro definition | 0 |
| `\|` or-pattern | not counted — see below |

**210 of 430 files (49%) contain at least one blocker.** Per crate:

| Crate | Files | Blocked | |
|---|---:|---:|---:|
| `redox-cli` | 26 | 22 | 85% |
| `redox-diag` | 20 | 15 | 75% |
| `redox-emit` | 79 | 58 | 73% |
| `redox-fmt` | 52 | 33 | 63% |
| `redox-driver` | 27 | 16 | 59% |
| `redox-lexer` | 32 | 16 | 50% |
| `redox-bench` | 14 | 6 | 43% |
| `redox-parser` | 104 | 32 | 31% |
| `redox-lsp` | 22 | 6 | 27% |
| `redox-ast` | 48 | 6 | 12% |
| `redox-fuzz` | 6 | 0 | 0% |

Two things about this table are worth more than the totals.

**Four of plan 46's seven gaps do not appear in the compiler at all.** `unsafe trait`,
`unsafe impl`, `extern "C" fn` and `macro_rules!` are zero. They are real gaps and they are
not *this* obstacle — which is a result only a count could give, and the opposite of what doc
02 §4.8 expected the attempt to find.

**One blocker is 94% of the problem.** 848 macro invocations against 9 casts. `format!`,
`write!`, `assert!`, `println!`, `vec!`, `matches!` and `panic!` are how this compiler
formats, asserts and reports; doc 18 §7 retired `!` as a suffix and left no spelling, so none
of them can be written.

**The measurement needed triaging, twice.** The first pass reported **22** casts; most were
`use … as …` **imports**, which Redox spells and which are not casts. Excluding import lines
brought it to 14, of which 5 are still false positives — a multi-line `use` whose `as` sits on
a continuation line, and the words *"as though"* inside a doc comment the stripper missed. The
honest number is **9**, and §6 records that the counter is line-based rather than a Rust
parser. Plan 46 §1.3's ratio, arriving on schedule in the row that cited it.

`|` or-patterns are **not counted**: no line-based regex separates a pattern alternative from a
bitwise or, and counting them wrongly is worse than leaving the cell empty. §6.

### 1.2 The answer, and what would change it

**No. The compiler's source cannot currently be expressed in Redox without the escape hatch.**

But the shape of the no is the finding. One construct — macro invocation — accounts for 848 of
857 blocking occurrences. If it were spelled:

- the blocked set drops to **9 `as` casts in 7 files**;
- `redox-fuzz` and `redox-ast` become portable almost entirely (0% and 12% blocked);
- the criterion becomes a question about two constructs rather than about the language.

So the useful output of this row is not a verdict, it is a **priority**: doc 03 §18 lists macro
invocation as open and plan 45 §1.5 declined to take it because it needs a spelling decision.
This row is the measurement that says how much that decision is worth — 94% of the distance to
a property doc 01 §34 made a project goal.

### 1.3 A worked port, because a count is not a proof

A count over regexes can be wrong in ways only compiling something reveals — §1.1 already found
it wrong twice. So leaf 2 ports one real compiler module to Redox and compiles the result.

`crates/redox-ast/src/program/item/clause.rs` is the subject: 45 lines of real compiler code,
a `struct` with four fields, derives, and doc comments carrying worked examples. **42 of
`redox-ast`'s 48 files are blocker-free**, and this is one of them.

What the port proves and what it does not:

- It proves a real compiler data type is expressible — declarations, fields, derives,
  visibility, doc comments and a verbatim type each survive the round trip.
- It does **not** prove the compiler is expressible. 45 of 24,159 lines is 0.2%, and §1.1's
  count is what speaks to the rest. The port's job is to show the count measures the right
  thing.

The generated `.xrs` is committed, per doc 02 §4.8 — *"Commit the generated `.xrs` so the
bootstrap never breaks"* — and rustc compiles it in the gate, because "expressible" means
"produces valid Rust" and nothing less.

### 1.4 The Rust implementation stays

Doc 02 §4.8 is explicit: *"keep the Rust implementation permanently rather than as a
'fallback'"*, correcting doc 01 §34's own wording. This row adds a `conformance/` directory and
changes no crate. Nothing in the build depends on the ported module, and the compiler is not
compiled from it.

Worth stating because the opposite reading — that a conformance port is the beginning of a
migration — would make every future plan carry two implementations.

---

## 2. Admission tests (doc 03 §2)

Not applicable: no construct, no lowering. A measurement, a ported module and a test.

---

## 3. Concrete proof this plan targets

```bash
moon run :test                                      # the count is current, the port compiles
REDOX_BLESS=1 cargo test -p redox-cli --test conformance
cargo run -p redox-cli -- check conformance/clause.sdx
```

The gate asserts:

1. `docs/CONFORMANCE.md` equals what the counter produces from the working tree — generated and
   blessable, as `docs/STATUS.md` is (plan 47 §1.2).
2. `conformance/clause.sdx` transpiles to the committed `conformance/clause.xrs`.
3. **rustc compiles that `.xrs`.** The claim is "expressible as valid Rust", and a golden alone
   would assert only that the emitter agrees with itself (plan 44 §3's argument).

Criterion 1 is what makes the row survive its own success: when macro invocation is spelled,
the count drops, the document fails, and whoever spelled it sees how much of the compiler they
just unblocked.

---

## 4. Leaves

### Leaf 1 — `leaf-conformance-count`

`crates/redox-cli/tests/conformance.rs` counting the blockers per crate and rendering
`docs/CONFORMANCE.md`, compared in the gate and blessable.

### Leaf 2 — `leaf-worked-port`

`conformance/clause.sdx` and the committed `conformance/clause.xrs`, with the test transpiling
and compiling it.

---

## 5. Acceptance criteria

1. `docs/CONFORMANCE.md` carries §1.1's per-blocker and per-crate counts, generated and
   gate-compared.
2. It states the answer to doc 02 §4.8's criterion **explicitly** — currently no — and names
   macro invocation as 94% of it.
3. `conformance/clause.sdx` is a port of a real compiler module, transpiles to a committed
   `.xrs`, and **rustc compiles it**.
4. The four zero-occurrence gaps are reported as zero rather than omitted, because "this gap
   does not block self-hosting" is the useful fact.
5. The `|` or-pattern cell says *not counted*, with the reason, rather than carrying a number
   the method cannot produce.
6. Nothing in the build depends on `conformance/`, and the document says the Rust
   implementation is permanent (§1.4).

Criteria 4 and 5 are both about reporting an absence honestly, which is plan 47's posture
applied to a measurement that has two kinds of hole.

---

## 6. Out of scope, and what it discloses

- **Porting the compiler.** 0.2% is ported and §1.1 counts the rest. Porting more would
  measure the same thing more slowly, and the one construct that blocks it is not this row's to
  spell.
- **Spelling macro invocation.** It needs a surface decision in doc 03 — doc 18 §7 retired `!`
  — and plan 45 §1.5 declined it for that reason. This row quantifies the decision; it does not
  take it.
- **A Rust parser for the counter.** The count is line-based regexes, which is why §1.1 had to
  triage twice and why the `|` cell is empty. A real parser would be `syn`, already in
  `Cargo.lock`, and would be the right tool if the count ever drives a decision finer than
  *"one construct is 94% of it"*. Named rather than pretended away.
- **Byte-identical round-tripping.** `conformance/clause.xrs` is not required to equal
  `clause.rs`. The claim is that the module is **expressible**, not that Redox reproduces a
  particular author's formatting, and requiring the latter would make every rustfmt change a
  conformance failure.
- **A bootstrap chain.** Doc 02 §4.8: *"`.xrs` is Rust, so there is no chicken-and-egg problem
  and no bootstrap chain to verify."* Committing the generated output is the whole of the
  bootstrap guarantee.

---

## 7. Gate

```bash
moon run :format :check :lint :build :test :audit
```

---

## 8. Emerald reuse

**None, and the register says why**: *"Emerald never self-hosted at 107k lines; weak evidence,
but the reframing argument is Redox-specific."*

The reframing is the transfer, and it is doc 02 §4.8's rather than Emerald's: a transpiler's
self-hosting is *nearly free and therefore nearly meaningless as a milestone*, so the thing
worth proving is expressiveness. Emerald's contribution is negative evidence — 203 plans
without the attempt — and this row's §1.1 turns the question into a number instead of a
milestone.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | Count the blockers across all 430 files | Port until something breaks | Plan 46 already found the gaps; porting would rediscover them one file at a time, and a count answers the criterion for every file |
| 2 | Port one real module anyway | Count only | A regex count can be wrong in ways only compiling reveals, and §1.1 found it wrong twice before the plan was written |
| 3 | `clause.rs`, 45 lines of `redox-ast` | A larger or synthetic module | Real compiler code, blocker-free, and exercising declarations, derives, visibility, doc comments and a verbatim type in one file |
| 4 | Generated and gate-compared, like `STATUS.md` | A written report | The count changes when the compiler does, and the row's value survives its own success only if the document fails when the number moves |
| 5 | Report the four zeroes | Omit them | *"This gap does not block self-hosting"* is the finding doc 02 §4.8 did not expect, and omitting a zero hides it |
| 6 | Leave `\|` or-patterns uncounted | Guess with a regex | No line-based pattern separates an alternative from a bitwise or, and a wrong number is worse than an empty cell |
| 7 | Do not require byte-identical output | Compare to `clause.rs` | The claim is expressibility; requiring one author's formatting would make a rustfmt change a conformance failure |
| 8 | The Rust implementation is permanent | Treat the port as step one of a migration | Doc 02 §4.8 corrects doc 01 §34's "fallback" wording, and the alternative makes every future plan carry two implementations |

---

## 10. Result

*Filled in when the plan ships, including anything above that turned out wrong.*
