# Plan 46 — Grammar Inventories

Plan ID: `grammar-inventories`
Depends on: 45 (`reachability-residue`)
Milestone: 7 — *tooling*
Status: `planned`

---

## 1. What this plan is

Doc 02 §4.1 asks for two graded inventories and says why the second one matters:

> **any Rust construct marked `UNREACHABLE` is a construct Redox users cannot write**, which
> directly violates §26's interoperability charter.

So the inventory is not documentation. It is **risk detection**, and doc 03 §18 is a summary
of one axis rather than a substitute for it — which this row has now measured, because §18
turned out to be wrong in one place and stale in another.

| Axis | Grades | This row |
|---|---|---|
| Rust's grammar | `PASSTHROUGH` / `SUGARED` / `UNREACHABLE` | the deliverable, with evidence per row and a test that the evidence resolves |
| MoonScript's grammar | `KEEP` / `MODIFY` / `REMOVE` | §1.5 — short, because doc 18 replaced the surface and doc 18 §10 calls this axis *"mostly moot"* |

### 1.1 Verified preconditions

Doc 02 §4.1's own candidate gap table is forty-five plans old, so it was re-run rather than
re-read: **26 probes in a first pass, 12 in a second and 10 in a third**, each a whole
program transpiled *and* compiled by rustc, because "it parses" is not the claim.

**Doc 02 §4.1's entire gap table is closed.** Attributes (plan 36), granular visibility,
`where` clauses, `impl Trait` (plan 25), match ergonomics, `loop` and labelled break (plan
20), `const`/`static`/`const fn` (plan 16), type aliases, `union` and `extern` blocks (plan
45), `mod` blocks (plan 14), `move` closures, raw/byte/char literals (plan 23), tuple and
unit structs — every row of it reachable, and each confirmed by a probe rather than by a
plan's own claim to have shipped it.

So the inventory's original purpose is discharged, and the row's value is what §1.2 found
instead.

**Two thirds of the first pass's failures were the probe's fault, not the language's.** That
ratio is the methodological finding of this row, recorded in §1.3, and it is why §4's leaf 2
exists.

### 1.2 Four gaps nobody had listed, and one claim that is false

Each survived three passes of re-spelling, and for each the *equivalent* was checked too — a
construct reachable by another spelling is `SUGARED`, not a gap.

**`as` casts are unreachable.** `n as u8` fails in a binding, as an argument and in a tail
expression; `as` is reserved for `import … as`. The nearest thing that works is
`u8::try_from n`, and it is **not** a cast: `as` truncates and `try_from` is fallible, so a
program that wants the truncating conversion cannot write it. Listed nowhere — not doc 02
§4.1's table, not doc 03 §18.

**`|` or-patterns are shadowed, and doc 03 §18 says otherwise.** §18 lists *"match
ergonomics — `@` bindings, `|` alternatives, slice patterns"* under **"Closed by this
document"**. `@` bindings and slice patterns are indeed reachable. `|` is not: doc 18 made
Haskell's surface the language, where `|` in a `case` arm introduces a **guard**. So
`1 | 2 -> true` parses — and means `1` guarded by `2`, which fails as *"mismatched types:
expected `bool`, found integer"*. A reader following §18 would write it and get a type error
about the wrong thing.

Expressiveness survives, which is why this is `SUGARED` and not a hole: separate arms match
the same values. What is lost is the *binding* case, where Rust's `|` shares one arm body.

**`unsafe class` and `unsafe instance` are unreachable** — Rust's `unsafe trait` and
`unsafe impl`. Plan 45 §6 named exactly this and said *"the spelling leaf 2 chooses should
extend to them; if it does not, §10 records that rather than this section promising it"*. It
does not, and **plan 45 §10 did not record it**. This row is the check plan 45 asked for,
one row later than intended.

**`extern "C"` on a function is unreachable.** Plan 45 leaf 3 shipped the *import* direction
— an `extern` block declaring foreign functions — and not the *export* direction, a Redox
function given the C ABI so C can call it. Doc 01 §26 names FFI without saying which way, and
half of it is still missing.

### 1.3 Two thirds of a careless inventory is wrong, which decides the row's shape

Of the first pass's 16 failures, **11 were the probe's fault**: `format` used as a function
when Redox has no macro-call syntax (plan 45 §1.5); `True` for `true`; `where t: Clone` when
Redox's `where` introduces *local bindings* and a bound is a clause read off `=>` (doc 18
§2.2); `@non_exhaustive` when attributes are written in Rust's own `#[…]`; an instance method
without the signature the instance must repeat; `a.0` on an array.

Every one of those would have been written down as `UNREACHABLE` by a less careful pass, and
each is worse than a missing row: it sends a future plan to implement something that already
works, and the plan would discover that only after designing a surface for it.

**That is the argument for leaf 2.** An inventory nobody checks rots in both directions — a
closed gap keeps reading as open, and a real gap quietly closes without the list noticing. So
each row of the Rust axis carries **evidence**: a corpus file that exercises it, or a fixture
that still fails. A test resolves the citations.

This is the same discipline plan 44 §3 used for its negative pair and plan 41 §1.3 used for
its corpus sweep: the artifact asserts something a machine can check, or it is prose.

### 1.4 What the grades mean here

Doc 02 §4.1 names three and leaves them to be defined against a real surface:

| Grade | Means | Evidence it carries |
|---|---|---|
| `PASSTHROUGH` | Redox writes the Rust construct essentially unchanged | a corpus file that uses it |
| `SUGARED` | reachable, but through a different spelling | the corpus file, and the Rust shape it lowers to |
| `UNREACHABLE` | a Redox program cannot express it | a fixture under `tests/unreachable/` that still fails, and the nearest thing that does work |

`UNREACHABLE` rows carry **the nearest working alternative** as well as the fixture, because
that is what a reader hitting the gap actually needs, and because writing it down is what
distinguishes a gap from an inconvenience — `as` has no alternative, `|` has one.

### 1.5 The MoonScript axis is a record, not a table

Doc 02 §4.1 asked for MoonScript graded `KEEP`/`MODIFY`/`REMOVE` against Emerald's 761-line
Ruby inventory. Doc 18 then **replaced the surface**: layout, equations, guards, `case … of`,
`data`/`newtype`/`class`/`instance` and the `::`/`->` type language are Haskell's, and doc 18
§10 records this axis as *"mostly moot"*.

Grading a surface that was discarded, production by production, would be archaeology. So the
MoonScript section is **one table of what survived and what replaced it**, which is the
question a reader actually has, and doc 18 §§2.1–2.7 is cited for the rest rather than
copied. §6 records the decline.

---

## 2. Admission tests (doc 03 §2)

Not applicable: no construct, no lowering. This row adds a document and a test.

---

## 3. Concrete proof this plan targets

```bash
moon run :test                                   # the citations resolve, the gaps still fail
cargo nextest run -p redox-cli --test grammar_inventory
```

The test asserts three things, and the third is the one that matters:

1. Every `PASSTHROUGH` and `SUGARED` row names a corpus file, and that file **exists**.
2. Every `UNREACHABLE` row names a fixture under `tests/unreachable/`, and that file exists.
3. **Every `UNREACHABLE` fixture still fails to compile.** When a later row closes a gap, the
   fixture starts passing and the test fails — which is the only way a list of gaps stays
   true without someone remembering to revisit it.

Criterion 3 inverts the usual direction: the test fails when the compiler gets *better*. That
is intended, and the failure message says so, because the alternative is a document that
claims a gap years after it closed.

---

## 4. Leaves

### Leaf 1 — `leaf-rust-axis`

`docs/GRAMMAR.md`'s Rust axis: every grammar area graded, each row carrying its evidence and
each `UNREACHABLE` row carrying the nearest working alternative. Organised by the Rust
reference's areas rather than by Redox's constructs, so a reader can check coverage against
the host language instead of against this project's history.

### Leaf 2 — `leaf-evidence-check`

`tests/unreachable/` with one fixture per gap, and
`crates/redox-cli/tests/grammar_inventory.rs` asserting §3's three properties.

### Leaf 3 — `leaf-moonscript-axis`

`docs/GRAMMAR.md`'s MoonScript section, per §1.5: what survived, what replaced it, and doc 18
cited for the rest.

---

## 5. Acceptance criteria

1. `docs/GRAMMAR.md` grades every area of Rust's grammar `PASSTHROUGH`, `SUGARED` or
   `UNREACHABLE`.
2. Every `PASSTHROUGH`/`SUGARED` row cites a corpus file that exists.
3. Every `UNREACHABLE` row cites a fixture that exists and **still fails**, and names the
   nearest working alternative.
4. The four gaps §1.2 found are in the inventory as `UNREACHABLE`, with fixtures.
5. Doc 03 §18's `|`-alternatives claim is **corrected** — it says "closed" and the construct
   is shadowed.
6. Plan 45's §10 is amended to record `unsafe class`/`unsafe instance`, which §6 of that plan
   said §10 would record.
7. The MoonScript axis is present as §1.5's record, and §6 says why it is not a table.
8. The test runs in the ordinary gate, not as a separate step.

Criteria 5 and 6 are corrections to shipped documents. A row whose job is an honest inventory
cannot leave two known-wrong claims standing in the documents it inventories.

---

## 6. Out of scope, and what it discloses

- **Closing the four gaps.** This row *finds* and records them; implementing `as` casts,
  or-patterns, `unsafe class` and the `extern` export direction is surface work with four
  separate spelling decisions, and bundling them into an inventory row would hide them.
  Recorded as `UNREACHABLE` with fixtures, which is what makes them findable.
- **A MoonScript production table.** §1.5. Doc 18 replaced the surface and doc 18 §10 calls
  the axis moot; grading a discarded grammar line by line is archaeology.
- **Grading Rust's *library*.** The inventory is about the grammar. `std` reachability is a
  different question with a different answer (paths work, macros do not), and plan 47's
  status document is where a gap table belongs.
- **Declines.** Doc 03 §22's declined-construct list is plan 47's `docs/DECLINED.md`. An
  `UNREACHABLE` row here means *cannot be written*; a decline means *deliberately not
  offered*, and conflating them would make both useless.
- **Checking that every corpus citation actually exercises its construct.** The test asserts
  the file exists, not that it contains the construct — asserting the latter needs a parse and
  a query per row, which is plan 41's grammar doing the inventory's job. Named rather than
  silently assumed, and the fixtures carry the stronger property.

---

## 7. Gate

```bash
moon run :format :check :lint :build :test :audit
```

---

## 8. Emerald reuse

`spec/GRAMMAR.md`'s **methodology**, which doc 02 §4.1 calls *"the highest-value design
transfer"* in the whole blueprint: a grammar is an inventory, graded per area, with one
example per kept row.

Two deliberate departures:

- **Evidence is machine-checked.** Emerald's inventory was prose with examples. §1.3's finding
  — that two thirds of a careless pass is wrong — is the argument for the test, and Emerald
  had no equivalent.
- **The second axis has no Emerald analogue at all.** Emerald compiled to nothing; Redox
  compiles to Rust, so `UNREACHABLE` is a category Emerald's inventory could not have had, and
  doc 02 §4.1 says it is *"where Redox's real risk lives"*.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | Evidence per row, checked by a test | Prose with examples, as Emerald had | §1.3: two thirds of the first pass was wrong, and a false `UNREACHABLE` sends a future row to build what exists |
| 2 | `UNREACHABLE` fixtures must still **fail** | Assert only that the fixture exists | A gap list that cannot notice a gap closing is wrong the moment the next row ships |
| 3 | The test fails when the compiler improves | Only fail on regressions | Intended, and the message says so; the alternative is a document claiming a gap years after it closed |
| 4 | Organise by Rust's grammar areas | By Redox's constructs | Coverage is a claim about the *host* language; indexing by Redox's own surface cannot show what is missing from it |
| 5 | `UNREACHABLE` rows name the nearest alternative | The grade alone | It is what a reader hitting the gap needs, and it distinguishes `as` (no alternative) from `\|` (separate arms) |
| 6 | MoonScript as a record, not a table | The full `KEEP`/`MODIFY`/`REMOVE` grading | Doc 18 replaced the surface; doc 18 §10 calls the axis moot (§1.5) |
| 7 | Correct doc 03 §18 and plan 45 §10 here | Leave them and note the discrepancy | §18's `\|`-alternatives claim is false and plan 45 §6 asked for the `unsafe class` check; an inventory row cannot leave known-wrong claims in what it inventories |
| 8 | Gaps are recorded, not closed | Close them in this row | Four spelling decisions bundled inside an inventory would hide four surface designs |

---

## 10. Result

*Filled in when the plan ships, including anything above that turned out wrong.*
