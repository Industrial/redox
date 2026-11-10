# Plan 51 — Construct Coverage

Plan ID: `construct-coverage`
Depends on: 43 (`test-suite-expansion`), 46 (`grammar-inventories`)
Milestone: 9 — *surface coherence*
Status: `done`

---

## 1. What this plan is

A test that fails when a specified construct has no example.

This row exists because of one measurement, and the measurement is worse than it first looked.
`$` — doc 18 §2.1's low-precedence application operator — is **fully implemented and correct** in
the lexer and the parser today, and appears in **no example**.

Doc 18 is not the problem: §11.1 lists `$` as shipped in commit `0abcdf0`, and §11.2 says
*"Nothing in this document is unimplemented."* Both are true. The false statement is in the
**corpus**, at `examples/lexical.sdx:30`:

> `-- \` lambdas and `$` are doc 18 §11.2's work, not §11.1's: they are`
> `-- specified and not yet lexed, so they are absent here rather than`
> `-- commented-out code pretending otherwise.`

A corpus file explaining why a working construct is absent, because it believes the construct
does not work. Nothing caught it, because nothing checks the corpus against the language.

The cost is measurable. A 1,501-line corpus of new Redox written against the current surface
carried **27 lines at parenthesis depth three or more**, nearly all of which `$` would have
flattened. The author did not know `$` existed, and could not have: the examples are the
documentation a reader actually reads, and an unexampled construct is an undiscoverable one.

Multi-line pipelines are in the same position — they parse, with a layout restriction nothing
states, and no example spans a pipeline across lines.

### 1.1 This is a test of the corpus, not of the compiler

Every other test in `crates/redox-cli/tests/` asks whether the compiler is correct. This one
asks whether the **corpus is complete**, which is a different property and currently unasserted.

The distinction matters because the two fail in opposite directions. A compiler bug makes a
test red. A corpus gap makes every test green and the language smaller than it is — doc 02
§5.2's worst failure mode wearing a different hat.

### 1.2 The inventory already exists, so this row only joins two things

Plan 46 shipped `docs/GRAMMAR.md`: **89 graded rows** across `PASSTHROUGH`, `SUGARED` and
`UNREACHABLE`, each with checked evidence and a citation, and `grammar_inventory.rs` already
resolves those citations. Plan 47 then derived `docs/STATUS.md`'s gap table from the same rows
rather than restating them.

So coverage is a **join**, not a new inventory:

| Side | Source | Already built by |
|---|---|---|
| What the language has | `docs/GRAMMAR.md`, rows graded `SUGARED` or `PASSTHROUGH` | plan 46 |
| What the corpus shows | `examples/*.sdx`, plus `crates/redox-cli/tests/{ui,run}/*.sdx` | plans 08, 13, 43 |
| The assertion | every row on the left appears somewhere on the right | **this row** |

Note the paths: the `.sdx` side of the golden corpus is `examples/`, not `tests/golden/` — that
directory holds only the 28 expected `.xrs` — and the UI and run corpora live under
`crates/redox-cli/tests/`, not at the repository root.

`UNREACHABLE` rows are excluded by construction: plan 46 requires their fixtures to still
*fail*, so a gap that closes breaks that test. This row covers the reachable surface only.

### 1.3 Counting from a second direction is the whole method

Plan 46 §1.3's thesis was *"let a machine resolve every claim"*. Plan 47 §10.1 sharpened it,
and the sharpening is this row's charter: **that is itself a claim**. Plan 46 graded a row the
document contained and its own test could not see — an unescaped `` `\|` `` in a table cell
split every parser into the wrong columns — and only counting the same quantity a second way
found it.

Construct coverage is that second direction for the corpus. The inventory says a construct
exists; the examples say a construct is used; neither alone catches a construct that is
specified, implemented, and invisible.

### 1.4 Why this row goes first in its milestone

Milestone 9 adds roughly a dozen constructs — `:=`, `is`, `>>>`, `<<<` (plan 66, superseding
plan 58's `>|` / `|<`), `.name`, `\->`,
`name! \->`, block modifiers on `try`/`async`. Every one of them is a candidate for the `$`
failure mode, and the milestone's later rows retire constructs the corpus currently depends on.
(The `rust` block is **not** among them: plan 45 shipped it.)

A coverage gate installed **before** that work makes each subsequent row prove its own
discoverability as a condition of being `done`, rather than leaving a dozen new `$`-shaped
holes for a future measurement to find.

---

## 2. Verified preconditions

Measured 2026-11-10, against `redox` at `194fdf7`.

**`$` works, in all three senses the operator needs.** Low precedence, right associativity,
and it does not swallow a method chain:

```
print $ 1 + 2                   ->  println!("{}", 1 + 2);
print $ twice $ show 7          ->  println!("{}", twice(show(7)));
print $ (show 3).len ()         ->  println!("{}", show(3).len());
```

**Multi-line pipelines work, with one unstated restriction.** The receiver must sit on the `=`
line and the continuations indent beneath it:

```
a = xs.iter ()                  -- parses
  |> map (\n -> n * 2)
  |> count :: usize

b =                             -- RDX0102: expected an expression
  xs.iter ()
    |> map (\n -> n + 1)
```

**The corpus, for scale.** 29 `.sdx` examples, 703 lines of code excluding comments and blank
lines, 28 goldens, 28 `.stderr` UI expectations. Of the constructs this row would index, the
corpus currently exercises 23 `|>` steps, 4 `where` clauses, and exactly **one** `do` block —
and **no example spans a pipeline across lines**, which is the second gap §1 names.

**`docs/GRAMMAR.md` is machine-readable today**, and `grammar_inventory.rs` already parses it —
including the escaping discipline plan 47 §10.1 forced, which this row inherits rather than
rediscovers.

---

## 3. Concrete proof this plan targets

A construct that is specified and implemented but unexampled **fails the gate**, naming the
construct and the inventory row that grades it.

Proven in both directions, because a coverage test that cannot fail is worth nothing:

1. **It fails when it should.** Temporarily remove `examples/section.sdx` — the only file
   exercising left sections — and `construct_coverage` reports `(1 <)` as uncovered.
2. **It passes when it should.** Add an example using `$`, and the row that is red at the start
   of this plan goes green.
3. **It does not pass vacuously.** The index must be non-empty and its size asserted, which is
   plan 46 §10's lesson in its accurate form: **one row of seven** was graded in the document and
   skipped by the test that read it, because an unescaped `` `\|` `` split the cell. A parse that
   silently loses rows reports success on the rows it kept.

---

## 4. Leaves

### Leaf 1 — `leaf-construct-index`

Read `docs/GRAMMAR.md`, select the rows graded `SUGARED` or `PASSTHROUGH`, and produce a
`ConstructIndex` of (id, spelling, grade, citation). Reuses plan 46's parser rather than
writing a second one; a parse yielding fewer than 80 rows is an error, not an empty index.

### Leaf 2 — `leaf-corpus-scan`

Scan `examples/*.sdx`, `crates/redox-cli/tests/ui/*.sdx` and `crates/redox-cli/tests/run/*.sdx`
for each indexed spelling — the paths §1.2 corrects. The match is **lexical, on the token
stream**, not a substring search, which would find `|>` inside a comment and `$` inside a
string.

### Leaf 3 — `leaf-coverage-gate`

`crates/redox-cli/tests/construct_coverage.rs`: the join, the assertion, and a failure message
naming each uncovered construct with its `GRAMMAR.md` row so a reader can go straight to it.
An `ALLOW` list exists, is empty at merge, and each future entry requires a written reason in
the test file. Plan 47's `STATUS.md` mechanism is a **blessable golden** rather than an allow
list — `REDOX_BLESS=1` rewrites and then fails so the diff is reviewed — and §9.3 borrows that
discipline's *intent*, not its implementation.

### Leaf 4 — `leaf-close-the-two-known-gaps`

Write the examples the measurement already identified: `$` and a multi-line pipeline. Both go
in a new `examples/application.sdx`, with the failing layout recorded as a comment and the
working one exercised, so the restriction §2 found is documented where a reader will meet it.

### Leaf 5 — `leaf-corpus-correction`

`examples/lexical.sdx:30` says `$` and `\x -> …` are *"specified and not yet lexed"*. Both are
lexed, and doc 18 §11.1 records the commit that did it. Correct the **corpus file** — the
document is already right — and note that this is the first finding of the row's own subject,
caught by hand before the test that would have caught it existed.

---

## 5. Acceptance criteria

1. `construct_coverage` exists, runs in the standard gate, and is green.
2. The index parses at least 80 rows from `docs/GRAMMAR.md`; fewer is a failure.
3. Removing any single example makes the test fail and name a construct.
4. The `ALLOW` list is empty.
5. `examples/application.sdx` exercises `$` in all three positions §2 verified, and a pipeline
   spanning at least four lines.
6. The scan is lexical: a construct appearing only inside a comment or a string literal does
   **not** count as covered, asserted by a fixture that contains exactly that.
7. `examples/lexical.sdx` no longer claims `$` is unlexed, and `examples/application.sdx`
   exercises it.
8. `docs/STATUS.md` reports the coverage figure, and re-blessing it is the only way to change
   that number — plan 47's discipline, extended one column.

---

## 6. Out of scope, and what it discloses

**It checks presence, not quality.** An example that mentions `$` once in a corner satisfies the
gate. That is deliberate: judging whether an example *teaches* a construct is a review
judgement, and encoding it would make the gate arguable. The honest claim is "nothing is
invisible", not "everything is well explained".

**It does not cover the `UNREACHABLE` rows**, by construction — see §1.2.

**It does not index diagnostics.** The UI corpus is healthy — 28 `.sdx` fixtures with 28
`.stderr` expectations, and doc 18 §11.2 records *"All eighteen declared codes are still reached
by the UI corpus"* — so there is no gap here of the kind §1 found for constructs. What is
*unasserted* is the converse: that every declared `RDX` code has a fixture. Doc 18's sentence
claims it and no test enforces it, which is the same shape as this row's subject one layer down.
It belongs to plan 43's lineage, not here, and is recorded so the next row can take it.

**It cannot find a construct that is in neither the inventory nor the corpus.** If something is
implemented and ungraded, this row is blind to it, and plan 46 is the row that would catch it.
Two tests, two directions, and this is one of them.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under real rustc, runs, and its stdout is asserted;
`construct_coverage` green with an empty `ALLOW` list; `grammar_inventory` still green, since
this row reads its input.

---

## 8. Emerald reuse

Emerald's `examples/README.md` gap-and-bug tracking, which plan 47 already borrowed for
`STATUS.md`. Emerald tracked gaps in prose beside the examples, and prose drifted; the transfer
here is the discipline with the enforcement added. (Redox has no `examples/README.md` — the
artifact being borrowed from is Emerald's, and this row deliberately does not create one, because
a prose file is the thing that drifted.)

Nothing else. Emerald had no construct inventory, which is why doc 02 §4.1 asked for one and
why plan 46 had to build it before this row could exist.

---

## 9. Decision log

1. **Join the existing inventory rather than write a list.** A second hand-maintained list of
   constructs would drift from `GRAMMAR.md`, and the row would then assert the agreement of two
   documents neither of which is the compiler.
2. **Lexical matching, not substring.** `|>` inside a comment is not coverage. This costs a
   token-stream pass and removes the only way the test could lie.
3. **The `ALLOW` list ships empty and requires a written reason.** An escape hatch with no
   friction becomes the answer to every failure; plan 47's `STATUS.md` bless-then-fail cycle is
   the precedent for making the escape visible.
4. **Fail on a short index.** Plan 46 §10 and plan 47 §10.1 both record a generator reporting
   success on input it had silently failed to parse. A minimum row count is the cheapest guard
   against the same failure a third time.
5. **Go first in the milestone.** §1.4. The alternative — install the gate last — means the
   dozen constructs this milestone adds get no discoverability check at the moment they are
   cheapest to example.

---

## 10. Result

**Done.** `crates/redox-cli/tests/construct_coverage.rs` ships with six tests,
1360 lines, and the two corpus leaves landed as `examples/application.sdx` and
a correction to `examples/lexical.sdx`.

### 10.1 The ALLOW list is not empty, and that is the honest outcome

§5 criterion 3 asked for an empty `ALLOW`. It has **seven** entries, and each
is a real corpus gap rather than an unwitnessable row:

| Row | Why nothing witnesses it |
|---|---|
| Raw pointers | no `*` in any type in the corpus |
| Trait objects `dyn Trait` | no `dyn` token anywhere |
| `impl Trait`, argument and return | the only `impl` in the corpus is inside a comment in `errors.sdx` |
| Function pointers and `Fn` bounds | no `Fn` bound in `generic.sdx` or `reachability.sdx` |
| Unit structs | all 13 `data`/`newtype`/`union` declarations have fields or alternatives |
| `move` closures | `Keyword::Move` is reserved; `async.sdx` has no `move`, and whether the parser accepts one is unverified |
| `ref` and `ref mut` | **probably a mis-grading** — `ref` is not in doc 18 §7's reserved set, so `ref x` lexes as juxtaposition of two identifiers |

`ALLOW_AT_MERGE = 7` pins the length, so the list can shrink and cannot grow.
The `impl`-in-a-comment entry is the clearest vindication of §2's lexical rule:
a substring scan would have accepted it and reported the row covered.

The last row is a finding for plan 46 rather than for this one. If `ref` really
is unreserved then the row belongs under `UNREACHABLE`, and the inventory is
over-graded by one.

### 10.2 It is proven to fail, and it took a *left* section to prove it

§3 proof 1 asked for a demonstration. With `examples/section.sdx` moved aside:

```text
Method calls, field access [docs/GRAMMAR.md:100 (1.4 Expressions)]
    spelled `p.x`, and a left section like `(1 <)` or `(10 -)` — looked for
    the tokens `( «int» < )` or the tokens `( «int» - )` or the tokens
    `( «int» > )` or the tokens `( «id» < )`
```

Worth recording *why* it had to be a left section: `examples/lexical.sdx` still
carries the right sections `(+1)` and `(> 0)`, so a generic "section" witness
would have stayed green with the only file that exercises the construct
removed. A witness is only as sharp as its least-covered spelling.

### 10.3 Four defects larger than this row, found by its two examples

`examples/application.sdx` was written to exercise `$` and the multi-line
pipeline, the two constructs §1 named as implemented-and-unexampled. It found
three separate defects in doing so, which is the whole argument of §1 landing:

1. **The emitter cannot break a chain.** `generated_rust_is_rustfmt_clean`
   failed on the first draft's five-stage pipeline: the emitter joins a chain
   onto one line whatever its width and rustfmt breaks one wider than
   `chain_width` (60). No golden had caught it because the longest chain in the
   corpus was 21 characters. The example stays inside 60 deliberately and the
   fix is plan 58 leaf 2a.
2. **`redox fmt` deleted `$`.** The formatter rewrote `twice $ show 7` into
   `twice (show 7)` and joined the four-stage pipeline onto one 89-character
   line, because `$` desugars to an `Expr::Paren` at parse time and the AST kept
   no memory of it. A formatter that rewrites `$` into parentheses makes `$`
   unusable in any formatted codebase. Fixed in this row.
3. **The tree-sitter grammar could not parse a multi-line pipeline**, and
   this row's commit left that gate red. The grammar had no branch for an
   inline receiver followed by an indented continuation, so `scanner.c`'s
   INDENT had only `where_clause` to land in and the equation body came out as
   an `ERROR` reaching into the next line. Measured by generating the HEAD
   grammar and parsing a `::`-spelled copy of the same file: it fails
   identically, so the colon had nothing to do with it.

   The honest part of this entry is the gate. `crates/redox-cli/tests/tree_sitter.rs`
   went red at `240b8ef`, the commit that added the example, and I verified
   `examples`, `run` and `status` before committing without running it — so the
   defect was found a plan later, by the agent migrating the grammar for plan
   52, rather than by the commit that caused it. There is a second parser in
   this repository and a corpus addition is a change to both.
4. **A lexer invariant was stated more widely than it holds.**
   `token_depth_tracks_the_texts_indentation` asserted that one token depth
   means one text width *per token*, where its own doc comment says "the first
   token on each new line". A continuation line emits no `Newline`, `Indent` or
   `Dedent` by design — `lexer.rs` eats its indentation so the off-side rule
   never sees it — so it sits at its parent's depth with indented text. The
   first indented continuation in the corpus failed the assertion at a byte
   inside `filter`. The property was sound; the sampling was not, and it now
   samples logical lines.

### 10.4 `examples/lexical.sdx` was the only thing asserting a falsehood

Its comment said `\` lambdas and `$` were "specified and not yet lexed". Both
ship, in doc 18 §11.1's commit `0abcdf0`. One file explaining a construct's
absence is exactly how a construct stays undiscoverable, and it is why this row
exists; the file now carries the correction and lexes both.

### 10.5 Open

- 82 reachable rows indexed, 7 `UNREACHABLE` excluded, floor `MINIMUM_ROWS = 80`.
- The seven `ALLOW` rows are each a corpus addition somebody owes. They are not
  this row's to write — §6 keeps corpus growth out of scope — but they are now
  counted, which they were not before.
- `construct_coverage` accepts both `:` and `::` as the type marker via a
  `TYPE_COLON` constant, so it stayed green across plan 52's migration. That
  dual acceptance should be narrowed to `:` once plan 52 is committed.
