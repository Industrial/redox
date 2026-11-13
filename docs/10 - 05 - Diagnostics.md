# Plan 05 — Diagnostics

Plan: 05 of 48 (`docs/04 - Voltaic Plan of Plans.md`, Milestone 2)
Slug: `diagnostics-miette`
Status: `planned`
Depends on: plan 03
Discloses: plans 06, 07, 08

---

## 1. What this plan is

Milestone 1 made the pipeline work. Milestone 2 makes it **explain itself**, and doc 02
§6.5 calls this the largest single gap in doc 01: §29 says "pass Rust errors through"
without a mechanism, so a user debugs generated `.xrs` they never wrote.

This plan does the first half — **Voltaic's own errors, rendered well**. The second half,
remapping *rustc's* errors back to `.sdx`, is plan 07 and depends on plan 06's span map.

Three deliverables:

1. the `voltaic-diag` crate, filling the empty shell plan 01 declared;
2. a **stable code catalogue** (`RDX####`), so every diagnostic is citable and doc 03
   §17's codes have a home;
3. `miette` rendering — a source snippet with a caret, not a one-line message.

Doc 01 §29's own example output is miette's format almost exactly, which is why doc 02 §5.5
names it. Emerald's inception §14.4 sets the same bar and states the principle this plan
exists to honour: *"Diagnostics are a first-class compiler feature, not an afterthought."*

### 1.1 The first third-party dependency

Voltaic has had **zero** third-party dependencies through four plans, and doc 06 §7 requires
a new one to carry a reason written down in the plan that adds it. This is that reason.

`miette` 7.6 is on doc 02 §5.5's shortlist of the four dependencies the project expects to
acquire, and it is the one with the least plausible alternative: rendering a source snippet
with a caret, a code, a help line and correct Unicode width is a few thousand lines of work
that is entirely incidental to Voltaic. Writing it would be the clearest possible case of the
"no Voltaic plan implements a library function" rule being violated in spirit.

Scoped deliberately: `miette` is a dependency of `voltaic-diag` and `voltaic-cli` only. No
stage crate gains it, because an error type needs a span and a message, not a renderer —
doc 06 §6 rule 2's division. If `voltaic-diag` ever needs to be swapped, the stage crates do
not notice.

---

## 2. Verified preconditions

`miette` 7.6.0 resolves and is fetchable in this environment (checked before authoring this
plan, so leaf 1 does not begin with a surprise).

Everything else this plan needs is already true: every `LexError` and `ParseError` carries
a span, because doc 06 §6 rule 3 made that mandatory from plan 01 and the enforcement is
that neither type compiles without one.

---

## 3. Concrete proof this plan targets

A UI-test corpus: each `tests/ui/*.sdx` is a file with one deliberate mistake, and each has
a committed `.stderr` beside it holding the exact rendering. Doc 01 §29's own example is the
bar:

```text
error[RDX0101]: expected `->` after the signature

  ⯈ bad_arrow.sdx:1:7
  1 │ add = (a: i32)
    ·       ▲
  2 │   a
```

Three properties the corpus asserts beyond "a message appears":

1. **the span points at the right place** — a caret under the offending token, not column 1;
2. **every diagnostic carries a code**, and the code is stable across runs;
3. **a file with three mistakes renders three diagnostics and exits non-zero once** — the
   per-error loop Emerald's `report_driver_error` establishes, which plan 03's two-level
   recovery already produces and nothing yet displays.

---

## 4. Leaves

### Leaf 1 — `leaf-code-catalogue`

`voltaic-diag`'s `Code` enum, with ranges chosen to respect what is already published rather
than renumbering it:

| Range | Holds |
|---|---|
| `RDX0001`–`RDX0039` | lexical errors — this plan assigns them |
| `RDX0040`–`RDX0099` | **reserved**: doc 03 §17 already assigned `RDX0042`–`RDX0063` to surface features from plans 19–28 |
| `RDX0100`–`RDX0199` | syntactic errors — this plan assigns them |
| `RDX1000`+ | lints, starting from doc 03 §17's `RDX1001` |

A test asserts every code is unique and in its declared range, so a collision with doc 03's
published numbers fails the build rather than shipping two meanings for one code.

### Leaf 2 — `leaf-diagnostic-type`

`voltaic-diag`'s `Diagnostic`: a code, a message, a span, an optional help line, and the
source it points into. Implements `miette::Diagnostic` so `{:?}` renders graphically — the
shape Emerald's `SemaError` uses (`crates/emerald-cli/src/main.rs:47-81`), which is the one
piece of its diagnostics work that transfers directly.

### Leaf 3 — `leaf-stage-errors-carry-codes`

`LexError` and `ParseError` gain a `code` field. Every construction site names its code, so
"which error is this?" is answered at the point the error is made rather than guessed at
render time by matching message text.

### Leaf 4 — `leaf-cli-rendering`

`voltaic-cli` renders through `miette` instead of `eprintln!`: one report per error, then a
single non-zero exit. A `--plain` flag keeps the unrendered form, because the UI tests need
output that does not depend on terminal width or colour support.

### Leaf 5 — `leaf-ui-test-harness`

`tests/ui/` with one `.sdx` and one `.stderr` per case, and `VOLTAIC_BLESS=1` regenerating
expectations exactly as the golden harness does. Cases cover at least: a tab, an
inconsistent dedent, an unterminated string, a missing `->`, a bad statement among good
ones, and a file with three independent mistakes.

### Leaf 6 — `leaf-tests-and-gate`

A completeness check — every `Code` variant appears in at least one UI case or is declared
unreachable — plus the full gate and independent re-verification.

---

## 5. Acceptance criteria

| # | Criterion |
|---|---|
| 1 | Every lexer and parser error carries a stable `RDX` code |
| 2 | Codes are unique and within their declared range |
| 3 | No code collides with doc 03 §17's published assignments |
| 4 | A diagnostic renders a source snippet with a caret at the right column |
| 5 | Three mistakes in one file render three diagnostics and exit non-zero **once** |
| 6 | `tests/ui/*.stderr` match byte for byte, and `VOLTAIC_BLESS=1` regenerates them |
| 7 | `--plain` output is free of colour and width dependence |
| 8 | `miette` is a dependency of `voltaic-diag` and `voltaic-cli` only |
| 9 | Plans 01–04's tests all still pass |
| 10 | Full gate green |

---

## 6. Out of scope, and what it discloses

| Deferred | To | Note |
|---|---|---|
| Remapping **rustc's** errors to `.sdx` | **plan 07** | Needs plan 06's span map; this plan renders only Voltaic's own errors |
| The span map | **plan 06** | Unchanged |
| A broader UI corpus as the grammar grows | each later plan | A plan that adds a diagnostic adds its UI case |
| `RDX1001`'s `unwrap` lint | **plan 38** | The range is reserved; no lint ships here |

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format
```

Plus `moon run :audit`, which matters for the first time now that a third-party dependency
exists.

---

## 8. Emerald reuse

**Lifted as a shape:** `crates/emerald-cli/src/main.rs:47-81` — `SemaError`'s
`miette::Diagnostic` implementation, which supplies `source_code` and one `LabeledSpan`,
and `report_driver_error`'s loop printing one report per recovered error before a single
non-zero exit. Doc 02 §3 names both as directly adaptable, and they are.

**Consulted:** Emerald's inception §14.4, for the bar and for the principle that
diagnostics are a first-class feature rather than an afterthought.

**Deliberately not taken:** Emerald's `thiserror` dependency. Voltaic's error types are three
structs with named fields; a derive macro to write their `Display` would be a dependency
earning less than its cost.

---

## 9. Decision log

- **`miette` is the first third-party dependency, with a written reason** (§1.1). It is on
  doc 02 §5.5's shortlist and it is the one whose absence would most clearly violate the
  "no plan implements a library function" rule in spirit.

- **`miette` does not reach the stage crates.** An error carries a span and a message; only
  the CLI renders. Doc 06 §6 rule 2's division, and it keeps the renderer swappable.

- **Code ranges respect doc 03 §17 rather than renumbering it.** Doc 03 published
  `RDX0042`–`RDX0063` for surface features from later plans. Taking `RDX0001`+ for lexical
  errors and `RDX0100`+ for syntactic ones leaves those untouched, and a uniqueness test
  makes a future collision a build failure.

- **Codes are assigned where an error is constructed, not where it is rendered.** The
  alternative — matching message text at render time — makes the message load-bearing, so
  improving a message silently changes a code.

- **`--plain` exists for the tests, not for users.** A UI corpus comparing bytes cannot
  depend on terminal width or colour detection. Making that a flag rather than an
  environment sniff keeps the test honest about what it is asserting.

---

## 10. Result

`done`. 162 tests pass; `fmt`, `clippy -D warnings` and `cargo audit` are clean.

| # | Criterion | Evidence |
|---|---|---|
| 1 | Every error carries a stable code | `LexError` and `ParseError` both have a non-optional `code: Code`; the type system, not a test, is what enforces it |
| 2 | Codes unique, within their range | `code.rs`: `every_code_is_unique`, `every_code_is_in_its_declared_range` |
| 3 | No collision with doc 03 §17 | `code.rs`: `no_code_collides_with_the_published_reservation` |
| 4 | Snippet with a caret at the right column | 20 `tests/ui/*.stderr` files, compared byte for byte |
| 5 | Three mistakes, three diagnostics, one non-zero exit | `tests/ui/three_errors.stderr`; `stderr_of` asserts the exit is non-zero and that stdout is empty |
| 6 | `.stderr` match byte for byte, `VOLTAIC_BLESS=1` regenerates | `ui.rs`: `every_ui_case_matches_its_expectation`, plus `no_expectation_is_orphaned` |
| 7 | `--plain` free of colour and width dependence | `plain_one.rs` writes ASCII only and consults no terminal; `parse_build_args` tests the flag in both positions |
| 8 | `miette` confined | `grep -l miette crates/*/Cargo.toml` → `voltaic-diag` alone. Tighter than the criterion asked: the CLI reaches `miette` only through `voltaic_diag::render` |
| 9 | Plans 01–04's tests still pass | Two assertions in `lex.rs` were *rewritten*, not weakened — see below |
| 10 | Full gate green | `moon run :check :lint :test :cargo-check :cargo-test :ci-format` and `:audit` |

### 10.1 What the corpus caught the moment it existed

The UI corpus is 20 programs and about 120 lines of harness. It found four defects in its
first run — every one of them in code that had been passing its tests for four plans:

1. **An escape error named the wrong line.** `scan_escape` runs over a cursor on the
   string's *body*, so `print "a \q b"` on line 2 reported line 1, column 3. The span was
   relative to the body and nothing rebased it. Fixed by `LexError::offset_by`, the same
   rebasing `lex_fragment` already did for tokens.

2. **The caret was counted in bytes.** A single `§` drew `^^`, because a span is a byte
   range while the column is counted in characters. Fixed by `caret_width`, which counts
   characters and tolerates a span that does not land on a boundary.

3. **A tab in the snippet pushed the caret out of step** — in the diagnostic whose whole
   subject is a tab. Fixed by expanding tabs to one space in the snippet line.

4. **`RDX0105` was declared and unreachable.** The empty-body error came out as a generic
   `RDX0101`, because the only thing that could raise `EmptyBody` was an indented-but-empty
   block the lexer cannot produce. Fixed by `Parser::expect_with`, which lets a specific
   expectation carry a specific code.

None of these is subtle. All four are the kind of thing that is invisible until something
prints the bytes and compares them, which is the argument for the corpus: not that it
proves the messages are *good*, but that it makes them *visible*.

### 10.2 Two deletions the corpus forced

Criterion 9 said plans 01–04's tests must pass. Two did not, and were rewritten rather
than relaxed — the distinction matters, so both are recorded:

- **`LexError` lost its `line` and `col` fields.** They duplicated the span, and defect 1
  above is what a duplicate looks like when it drifts: the span was right the whole time
  and the denormalised pair was wrong. `lex.rs`'s two position assertions now read the
  span, which is a *stronger* claim — `&src[err.span.0..err.span.1] == "\t"` says the
  error covers the tab, where `err.col == 1` only said it claimed to.

- **`Cursor` lost its line tracking entirely.** Once no error read a line number, the
  `line`/`line_start` fields and the `line()`/`col()` accessors served only their own
  tests, and `mark`/`reset_to` collapsed from a three-tuple to a `usize`. The computation
  now exists once, in `voltaic_diag::line_col`. Doc 06 §3 gained rule 6 for this.

### 10.3 Disclosures

- **`1e` is still accepted by the lexer.** It lexes as `Int("1e")` — `e` is swallowed as a
  type suffix — and rustc rejects it later. Validating suffixes would mean maintaining
  rustc's suffix list, which contradicts the deliberate choice (plan 02) that Voltaic never
  agrees with rustc about numeric detail. Recorded, not fixed; the `MalformedNumber` code
  is reached by `0x` instead.

- **Recovery quality is visible now, and it is mediocre.** `three_errors.stderr` opens with
  "expected a line break after the statement" where "expected an expression" is what
  happened; the first error consumed the blank line separating two items. That is plan 03's
  recovery, not plan 05's rendering, and the corpus is now where such a thing gets noticed.
  The `.stderr` file will change when recovery improves, and that change will be reviewable.

- **`miette` brings 68 crates into `Cargo.lock`.** `cargo audit` reports no advisories
  against any of them. The number is worth stating plainly for a project whose dependency
  count was zero one plan ago.

- **`"""` block interpolation spans are approximate.** A dedented block's body offsets no
  longer correspond one-to-one with the file, so an escape error inside one points into the
  right region rather than at the exact character. Plan 06's span map is the fix.
