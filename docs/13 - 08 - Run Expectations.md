# Plan 08 — Run Expectations

Plan ID: `run-expectations`
Depends on: 05 (`diagnostics-miette`)
Milestone: 2 — *rustc's errors land on the user's source*
Status: `done`

---

## 1. What this plan is

The third leg of Voltaic's test corpus: **what a program prints when it runs**, kept in
files beside the `.sdx`, blessable, with no expectation written as a Rust string literal.

Voltaic now has three distinct things to assert about a program, and had file-based
corpora for only two:

| What it asserts | Corpus | Since |
|---|---|---|
| The generated text, exactly | `tests/golden/*.xrs` | plan 01 |
| The diagnostic, exactly | `crates/voltaic-cli/tests/ui/*.stderr` | plan 05 |
| **What it prints when run** | **`crates/voltaic-cli/tests/run/*.stdout`** | **this plan** |

Doc 04 listed plan 08 as `ui-test-harness`. Plan 05 delivered that harness early — the
`.sdx`/`.stderr` pairs, `VOLTAIC_BLESS=1`, and `code_coverage.rs`. What was left over is
this, and §2.1 explains why the leftover turned out to matter more than the original
framing suggested.

### 1.1 Why a run corpus is not redundant with the goldens

A golden file proves the emitter produced the bytes it produced last time. It cannot prove
those bytes mean the right thing. Doc 02 §8.1–8.2 makes the point and plan 03 acted on it
by asserting execution as well as text — `phase1.sdx` prints `30` through real rustc, and
that assertion has caught a defect a golden could not: the missing tail expression, where
`a + b;` was textually plausible and semantically `()`.

Precedence is the sharpest case. `2 + 3 * 4` has a *correct answer*, and `14` distinguishes
a right tree from a wrong one far better than any amount of staring at a parenthesised dump
does. A run corpus is where arithmetic does the reviewing.

---

## 2. Verified preconditions

| Claim | How checked | Result |
|---|---|---|
| Execution is already asserted somewhere | `tests/phase1.rs`, `tests/hello.rs` | Yes — `support::Case::run()` compiles with real rustc and returns stdout |
| …but the expectations are Rust string literals | `phase1.rs:20`, `phase1.rs:29-42` | `assert_eq!(case.run(), "30\n")`, and a ten-line `concat!` of expected numbers |
| …and so is a generated-Rust expectation | `phase1.rs:16-18` | The whole `.xrs` as an escaped literal, duplicating `tests/golden/phase1.xrs` |
| A blessable corpus mechanism exists to copy | `tests/ui.rs` (plan 05) | `VOLTAIC_BLESS=1`, orphan detection, exact comparison |
| `Case::run()` asserts zero stderr | `support.rs` | Yes — a rustc warning on generated code is treated as an emitter defect |

### 2.1 The precondition that failed: the rule was already being broken

AGENTS.md states, under *conventions that are load-bearing, not stylistic*:

> **Every `.sdx` fixture is a file, and tests reference files.** Never embed language source
> as a string literal in a Rust test. Emerald's one syntax cutover missed 113 fixtures that
> lived as string literals inside test modules; that cost is paid once per syntax change,
> forever.

The `.sdx` sources do live in files. The **expectations do not**, and the rule's own
rationale applies to them with equal force. When the emitter's output changes, every
literal in `phase1.rs` must be found and hand-edited; a `.stdout` or `.xrs` file is
re-blessed and reviewed as a diff. The 113-fixture cost Emerald paid is not specific to
*source* text — it is the cost of test data that only a human editing Rust can update.

`phase1.rs:16-18` is the clearest instance: it duplicates `tests/golden/phase1.xrs` as an
escaped string literal, so the same fact is asserted twice, in two formats, and the two
can disagree. One of them is blessable and one is not.

So this plan does two things rather than one: it adds the run corpus, and it moves the
existing expectations into it. The second half is the part that pays off immediately.

---

## 3. Concrete proof this plan targets

1. **Every `tests/run/*.sdx` compiles with real rustc, runs, and prints exactly its
   `.stdout`** — byte for byte, with `VOLTAIC_BLESS=1` to re-record deliberately.
2. **No expectation in `crates/voltaic-cli/tests/` is a multi-line string literal of
   generated Rust or program output.** Machine-checked, so the rule cannot erode the way
   it already did.
3. `phase1.sdx`'s `30` and `expressions.sdx`'s ten hand-computed values survive the move
   with their reasoning intact — the comments explaining *why* `2 + 3 * 4` is `14` are the
   valuable part and must not be lost in the translation to a `.stdout` file.

---

## 4. Leaves

### Leaf 1 — `leaf-run-corpus`

`crates/voltaic-cli/tests/run.rs` plus `tests/run/`. Each case is a pair:

```text
tests/run/precedence.sdx       the program
tests/run/precedence.stdout    what it prints
```

Modelled on plan 05's `ui.rs` deliberately: same blessing flag, same orphan check, same
exact comparison. A second mechanism for the same job would be a second thing to learn.

### Leaf 2 — `leaf-annotated-expectations`

A `.stdout` file cannot hold a comment, and plan 03's `# 2 + 3 * 4, not 20` comments are
the reason the corpus is worth anything. They move into the **`.sdx`**, where they belong:
the program says what it is computing, and the `.stdout` says what came out. Comments
reach the generated `.xrs` anyway (doc 02 §10.1), so this costs nothing and documents the
case at the only place a reader will look.

### Leaf 3 — `leaf-migrate-literals`

`phase1.rs`'s three literal expectations move: the generated-Rust literal is deleted
outright (the golden already asserts it), and the two output literals become `.stdout`
files. `phase1.rs` keeps the assertions a corpus cannot express — the type round-trip and
the `->`-after-`)` refinement, which assert *shape*, not output.

### Leaf 4 — `leaf-no-literal-expectations`

`crates/voltaic-cli/tests/no_literal_expectations.rs`. Fails if a test file contains a
multi-line string literal that looks like generated Rust or program output. Scoped
tightly, with its heuristic stated in the test, because a check that fires on prose is a
check people disable.

### Leaf 5 — `leaf-tests-and-gate`

Gate, doc 04 status, doc 06 §10 rows.

---

## 5. Acceptance criteria

| # | Criterion |
|---|---|
| 1 | Every `tests/run/*.sdx` compiles, runs, and prints exactly its `.stdout` |
| 2 | `VOLTAIC_BLESS=1` re-records them; without it a mismatch fails |
| 3 | No `.stdout` is orphaned, and no `.sdx` lacks one |
| 4 | Each case's reasoning lives in the `.sdx` as comments, and reaches the `.xrs` |
| 5 | `phase1.rs` holds no multi-line expectation literal, and the behaviour it asserted is still asserted |
| 6 | A machine check fails if such a literal is reintroduced |
| 7 | Full gate green |

---

## 6. Out of scope, and what it discloses

| Deferred | To | Note |
|---|---|---|
| Multi-file cases | the plan that adds modules | A multi-file corpus needs a notion of a second file to import. Voltaic has one translation unit today, so a multi-file case could only test the harness |
| A `voltaic test` subcommand | **rejected, not deferred** | See §6.1 |
| Expected **exit codes** | — | Every case exits zero today, because `Case::run()` asserts it. A case that must exit non-zero needs a field in the corpus format, and no construct produces one yet |
| stdin | — | No construct reads it |

### 6.1 `voltaic test` is rejected

Doc 04's note on this plan proposed *"a `voltaic test` entry point so the corpus is runnable
without cargo."* That contradicts doc 01 §27 and plan 11, both of which fix the CLI at
`build`, `check`, `fmt`, `watch`, `--version`, `--help` — *"Cargo owns build, test, docs
and dependencies."*

The corpus is a cargo integration test, and `cargo test -p voltaic-cli --test run` runs it.
Adding a subcommand would duplicate a test runner Voltaic has, for an audience that already
has cargo installed by definition — `voltaic check` cannot work without it. Doc 04's row is
corrected rather than implemented.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format
```

---

## 8. Emerald reuse

**Lifted as a lesson, at full strength:** Emerald's 113 string-literal fixtures, which its
one syntax change could not reach. AGENTS.md already records the lesson; this plan is the
first time it has been applied to *expectations* rather than sources, and §2.1 is the
finding that it needed to be.

**Not applicable:** Emerald's own test layout. It tests a compiled language with its own
runtime; Voltaic's run corpus exists because rustc will run the output, which Emerald has no
analogue for.

---

## 9. Decision log

- **The run corpus copies `ui.rs`'s mechanism exactly** — same bless flag, same orphan
  check. Two corpora with two idioms would be two things to learn for no gain.

- **Reasoning moves into the `.sdx`, not into the test file.** A `.stdout` cannot carry a
  comment, and the explanation of why `2 + 3 * 4` is `14` is the part a reviewer needs.
  Putting it in the program also exercises trivia preservation on every run.

- **The duplicated generated-Rust literal is deleted, not moved.** `tests/golden/` already
  asserts it, blessably. Two assertions of one fact in two formats is one assertion and one
  liability.

- **The machine check is heuristic and says so.** It looks for multi-line literals
  containing `fn ` or a run of `\n`-terminated values in the test directory. A precise
  check would need to parse Rust; a loose one that people silence is worse than a stated
  heuristic with an explicit allow.

- **`voltaic test` is rejected on doc 01 §27 grounds** (§6.1), and doc 04's row corrected.

---

## 10. Result

`done`. 249 tests pass; `fmt`, `clippy -D warnings` and the full gate are green.

| # | Criterion | Evidence |
|---|---|---|
| 1 | Every case compiles, runs, prints exactly its `.stdout` | `run.rs`: `every_run_case_prints_its_expectation`, over four examples and two corpus cases |
| 2 | `VOLTAIC_BLESS=1` re-records; a mismatch otherwise fails | same test; the bless path writes and continues, the plain path compares |
| 3 | No orphans in either direction | `every_case_is_a_complete_pair`, plus `the_corpus_holds_only_cases_and_expectations` |
| 4 | Reasoning in the `.sdx`, reaching the `.xrs` | `every_case_documents_itself_and_keeps_its_comments` |
| 5 | `phase1.rs` holds no expectation literal | rewritten; the two shape assertions remain, the three data assertions moved |
| 6 | A machine check catches reintroduction | `no_literal_expectations.rs`, including a test that the heuristic actually fires |
| 7 | Full gate green | `moon run :check :lint :test :cargo-check :cargo-test` |

The corpus, with the expectation beside each program:

```text
examples/hello.sdx          examples/hello.stdout
examples/phase1.sdx         examples/phase1.stdout
examples/expressions.sdx    examples/expressions.stdout
examples/comments.sdx       examples/comments.stdout
tests/run/literals.sdx      tests/run/literals.stdout
tests/run/strings.sdx       tests/run/strings.stdout
```

### 10.1 Two emitter defects the corpus caught on its first run

Both were in string handling, both had been shipping since plan 02, and neither was
reachable by any existing test — the goldens compared Voltaic's output to Voltaic's output, and
no example contained the characters involved.

**`\#` reached rustc and rustc rejected it.** `\#` is Voltaic's one escape that Rust does not
have: it escapes the `#` that would otherwise open an interpolation. The parser carries a
literal's *raw* text to the emitter, which is correct for every escape the two languages
share and wrong for exactly this one, so `print "brace: \#{"` produced

```text
error: unknown character escape: `#`
```

Fixed by `emit/rust_escapes.rs`, which translates that single escape and leaves the rest
alone. Re-encoding the whole literal from `parts` was the alternative and was rejected:
it would mean Voltaic re-encoding every escape Rust already understands, each one a chance
to get a character wrong in a way no test would notice.

**A brace in a literal became a format placeholder.** `print "see {this}"` lowered to
`println!("see {this}")`, where `{this}` names a variable that does not exist. The user
would have got a rustc error about an identifier they never wrote. Fixed by
`emit/double_braces.rs`.

The two are deliberately separate functions, and the distinction is load-bearing:
`rust_escapes` makes a literal valid *Rust* and applies wherever one is emitted;
`double_braces` makes a literal mean itself as a *format string* and applies only where
one is used as that. `let s = "{}"` must keep its braces single.

`print_call` needed both and had neither, because it reads `StrLit`'s value directly
rather than going through `emit_expr` — the duplicate-path problem doc 06 §3 rule 6 names,
found again.

Worth stating plainly: **no golden file changed.** The only golden diff in this plan is the
two comment lines added to `examples/phase1.sdx`. Both fixes left every already-correct
example byte-identical, which is the evidence that they are narrow.

### 10.2 Two fixture violations, one of them an hour old

`no_literal_expectations.rs` was written to stop the rule eroding again. It immediately
found two places where it already had:

- **`hello.rs` embedded the whole generated `.xrs`** as an escaped literal, duplicating
  `tests/golden/hello.xrs`. The same fact asserted twice in two formats, one blessable and
  one not. Deleted; the golden keeps it.
- **`check.rs` embedded four `.sdx` programs** as literals — written by plan 07, earlier
  the same session, by me. That is the original AGENTS.md rule and not merely its
  expectation corollary: four programs a syntax change would have to find inside Rust.
  Moved to `tests/check/*.sdx`, where each one also explains which case it is.

Moving them promptly broke three of plan 07's assertions, because each fixture gained a
comment header and the line numbers its assertions name are literals. That is the correct
behaviour of a test that writes positions out by hand, and `source_map.rs`'s
`phase1_tail_expression_maps_to_its_source_line` failed for the same reason and the same
reason it should: a test that computed the line from the file would have passed while
proving nothing.

### 10.3 Disclosures

- **The check is a heuristic.** It looks for multi-line literals containing `fn ` with a
  brace, or three or more `\n`-terminated values. A precise check would have to parse
  Rust. A test asserts the heuristic fires on the patterns it claims to catch and not on
  an ordinary substring assertion, so it cannot pass by being blind. One allowance exists,
  for this file, with its reason.

- **Two corpus cases were premature and are now documented as such.** `print -3 + 10`
  parses as subtraction with `print` on the left, because the whitespace-adjacency rule
  doc 03 §5.4 specifies is plan 19's; `starts_argument` names plan 19 as the owner. And
  `#{...}` is literal text until plan 23. Both cases were rewritten to what works today
  with a comment naming the plan that will extend them — which is better than deleting
  them, because the comment is now the record of what is missing.

- **`examples/expressions.sdx` already reached the same conclusion about `(-3)`
  independently**, in a comment written by plan 03. Finding that after writing a duplicate
  precedence case is what prompted the decision that expectations live *beside the
  program*: the duplicate would have drifted.

- **`voltaic test` was rejected rather than deferred** (§6.1), and doc 04's row for this plan
  corrected. Doc 01 §27 fixes the CLI surface, and the corpus is a cargo integration test.

- **Multi-file cases remain out of scope** and will stay so until there is a second
  translation unit to import. A multi-file case today could only test the harness.
