# Plan 04 — Emitter Determinism

Plan: 04 of 48 (`docs/04 - Redox Plan of Plans.md`, Milestone 1)
Slug: `emitter-determinism`
Status: **`done`** — shipped 2026-11-08; see §10 for evidence per criterion
Depends on: plan 03
Discloses: plans 06, 34

---

## 1. What this plan is

Plan 01 established three emitter disciplines — deterministic output, no rustfmt in the
contract, line-preserving emission — and plan 03 grew the emitter to cover every AST node.
This plan makes those disciplines **checked rather than intended**, and closes the gap that
opened while the language grew: four examples exist and only two have golden files.

Doc 01 §3.4 requires output that is "deterministic and readable". Determinism is easy to
assert and was. **Readability was never checked at all**, and this plan is mostly about
finding a way to check it that does not smuggle rustfmt into the contract.

### 1.1 Readability, checked without putting rustfmt in the contract

Doc 02 §5.3 forbids running rustfmt as part of emission, for three reasons that still
stand: output would depend on an external tool's version, determinism would break, and the
span map plan 06 needs would be destroyed.

None of that forbids using rustfmt as a **measurement**. "Would rustfmt change our output?"
is a precise, mechanical definition of readable Rust, and it costs nothing at emit time
because it runs in a test. So:

> **The emitter's output must already be rustfmt-clean.** A test runs rustfmt over each
> generated `.xrs` and asserts it changes nothing.

That is a strictly stronger claim than "we format it ourselves and hope", and it is what
makes doc 01 §3.4's "readable" an assertion instead of an aspiration. If rustfmt disagrees,
the emitter is wrong — not the test.

---

## 2. Verified preconditions

Plan 01 §2 established that rustc compiles a `.xrs` directly. This plan additionally needs
`rustfmt` to be available and to accept a `.xrs` extension; §4 leaf 3 verifies both before
relying on them, and skips loudly rather than silently if rustfmt is absent (the pattern
plan 02's `AGENTS.md` gotcha prescribes).

---

## 3. Concrete proof this plan targets

Every example in `examples/` satisfies all five:

1. it transpiles;
2. its output matches a golden file byte for byte;
3. **rustfmt changes nothing** about the generated Rust;
4. rustc compiles it with zero stderr;
5. transpiling twice produces identical bytes.

Plus a completeness check: **an example with no golden file is a test failure**, so a new
example cannot silently skip coverage.

---

## 4. Leaves

### Leaf 1 — `leaf-golden-completeness`

`tests/golden/` currently holds `hello` and `comments`; `phase1`, `expressions` and
`lexical` have none. Rather than copy fixtures into a second directory — which plan 01 did
and which guarantees drift — the golden harness reads **`examples/` directly** and keeps
only the expected `.xrs` beside it under `tests/golden/`.

A test asserts every `examples/*.sdx` has a golden expectation. `lexical.sdx` is the one
exception and must be declared as such: it exercises the lexer and does not parse, so it
carries a documented exemption rather than an unexplained absence.

### Leaf 2 — `leaf-determinism-across-every-example`

Plan 01 asserted byte-identical output for one example. This extends it to all of them, and
adds the structural guarantee behind it: **`redox-emit` contains no `HashMap` or `HashSet`**,
whose iteration order is unspecified. A grep-style test over the crate's own sources, in the
spirit of doc 06 §10's enforcement.

### Leaf 3 — `leaf-rustfmt-clean`

§1.1's check. Resolve rustfmt by the same verify-don't-trust rule `build.rs` already applies
to rustc — plan 03's third blaze shim is the reason that rule exists — then for each
generated `.xrs`, run `rustfmt --emit stdout` and assert the result equals the input.

### Leaf 4 — `leaf-emitter-fixes`

Whatever leaf 3 finds. The expectation is that blank-line placement and operator spacing
need work; the point of the check is that this list is discovered rather than guessed.

### Leaf 5 — `leaf-tests-and-gate`

One harness driving all five properties per example, so adding an example adds five
assertions. Full gate, then independent re-verification.

---

## 5. Acceptance criteria

| # | Criterion |
|---|---|
| 1 | Every `examples/*.sdx` has a golden `.xrs`, or a declared exemption |
| 2 | Every example's output matches its golden file byte for byte |
| 3 | **rustfmt changes nothing** about any generated `.xrs` |
| 4 | Every parseable example compiles under rustc with zero stderr and runs |
| 5 | Transpiling any example twice produces identical bytes |
| 6 | `redox-emit` contains no `HashMap`/`HashSet` |
| 7 | `REDOX_BLESS=1` regenerates goldens and then fails the run |
| 8 | Plans 01–03's tests all still pass |
| 9 | Doc 06 §§1–2 obeyed |
| 10 | Full gate green |

---

## 6. Out of scope, and what it discloses

| Deferred | To | Note |
|---|---|---|
| The span map | **plan 06** | This plan keeps line-preserving emission; it writes no map |
| A canonical `.sdx` formatter | **plan 34** | This plan formats the *output*, not the input |
| `miette` rendering | **plan 05** | Unchanged |
| An `--rustfmt` opt-in flag | **plan 34** | Doc 02 §5.3 allows one later, marked as invalidating the map |

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format
```

---

## 8. Emerald reuse

**None.** Emerald emits machine code through LLVM, so it has no analogue of a readable
text artifact to keep rustfmt-clean. Its `benchmarks/REPORT.md` methodology discipline is
the nearest relative and belongs to plan 44, not here.

---

## 9. Decision log

- **rustfmt is a measurement, not a contract** (§1.1). Doc 02 §5.3's three objections are
  all about *emission time*; none applies to a test. Using it as a check turns doc 01 §3.4's
  "readable" from an aspiration into an assertion, and makes the emitter — not the test —
  wrong when they disagree.

- **The golden harness reads `examples/` rather than a copy.** Plan 01 copied each fixture
  into `tests/golden/`, which means two files to keep in step and a silent divergence when
  one is edited. Only the *expectation* belongs in `tests/golden/`.

- **A missing golden file is a failure, not an omission.** Four examples existed and two had
  coverage, which is exactly how coverage rots. Making absence loud costs one test.

- **No `HashMap` in the emitter is enforced, not just intended.** Determinism asserted on
  today's examples would not catch a future `HashMap` whose iteration happens to be stable
  for small inputs. The structural check catches the cause rather than one symptom.

---

## 10. Shipped — evidence per criterion

Executed 2026-11-08. Full gate green. **137 tests pass**, up from 132, and the five that
left are plan 01's superseded golden harness — see the deviations below.

All ten criteria are carried by one harness, `crates/redox-cli/tests/examples.rs`:

| # | Criterion | Evidence |
|---|---|---|
| 1 | Every example has a golden, or a declared exemption | `every_example_has_a_golden_expectation` |
| 2 | Golden match, byte for byte | `every_example_matches_its_golden_and_is_deterministic` |
| 3 | **rustfmt changes nothing** | `generated_rust_is_rustfmt_clean` |
| 4 | Compiles with zero stderr, and runs | `every_example_compiles_and_runs` |
| 5 | Byte-identical on repetition | same test as criterion 2 |
| 6 | No `HashMap`/`HashSet` in the emitter | `the_emitter_contains_no_unordered_collection` |
| 7 | `REDOX_BLESS=1` rewrites then fails | exercised live while blessing `phase1` and `expressions` |
| 8 | Plans 01–03's tests still pass | whole suite green |
| 9 | Doc 06 §§1–2 obeyed | `one_construct_per_file.rs`, `directory_structure.rs` |
| 10 | Full gate green | six moon tasks, exit 0 |

### The readability check's result

**Every example was already rustfmt-clean.** §1.1 expected leaf 4 to find spacing and
blank-line defects; it found none, so the emitter's own formatting needed no change. That
is a better outcome than the plan predicted, and it means doc 01 §3.4's "readable" is now
an assertion rather than an aspiration.

Getting there required one correction, found by the probe rather than by reasoning: run
rustfmt **with its defaults, outside the repository**. Inside the tree, rustfmt picks up
this repo's own `rustfmt.toml` — two-space indentation, which governs Redox's *source* —
and every example "failed" purely on indentation against output that deliberately uses
rustfmt's default four. The check was comparing the wrong two things.

### Three defects found by running it, not by inspection

1. **Scratch directories collided between parallel tests.** Keyed on tag plus pid, two
   concurrent tests shared a directory and one test's `Case::drop` deleted the `.xrs`
   another was still compiling — surfacing as *"couldn't read …/comments.xrs"*. Now keyed
   by an atomic counter as well.
2. **The `HashMap` check flagged its own prose.** `emit.rs`'s doc comment explains *why* the
   emitter uses no `HashMap`, and matching the bare word counted that as a violation. Now
   skips comment lines and matches usage (`HashMap<`, `HashMap::`) rather than mention.
3. **Both new checks were verified to bite.** An injected example without a golden, and an
   injected `HashMap` in `expr_list.rs`, each failed with a precise message naming the file
   before being reverted.

### Deviations from the plan as written

- **Plan 01's `golden.rs` was deleted, not extended.** It read its case list from `.sdx`
  files *copied* into `tests/golden/`, which is the drift the decision log predicted: two
  copies of every fixture to keep in step. `examples.rs` reads `examples/` directly and
  `tests/golden/` now holds only expectations. The copies are gone.
- **Leaf 4 produced no emitter changes**, because leaf 3 found nothing to fix. Recorded
  rather than quietly dropped: the leaf existed to hold whatever the check found, and the
  honest answer is that it found the emitter already correct.
- **`lexical.sdx`'s exemption carries its own test.** Rather than a bare entry in a skip
  list, `the_exempt_example_lexes_but_does_not_parse` asserts it still lexes *and* still
  fails to parse, so the exemption cannot hide a regression in either direction and must be
  removed deliberately once a grammar accepts it.

### What Milestone 2 inherits

Milestone 1 is complete: a program goes from `.sdx` to a running binary, the pipeline's
output is deterministic and rustfmt-clean, and every example is held to five properties at
once. Line-preserving emission is established and unbroken, which is the precondition plan
06's span map depends on.
