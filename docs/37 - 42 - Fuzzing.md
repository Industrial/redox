# Plan 42 — Fuzzing

Plan ID: `fuzzing`
Depends on: 04 (`emitter-determinism`), 43 (`test-suite-expansion`) — see §1.3
Milestone: 7 — *tooling*
Status: `planned`

---

## 1. What this plan is

Four fuzz targets — `lex_source`, `parse_program`, `emit_program`, and a generated-program
target whose oracle is **the generated Rust must parse**.

Doc 04's row calls that last one *"a stronger oracle than Emerald had"*, and it is: Emerald's
fuzzing asserted *does not panic*, which finds crashes and nothing else. An oracle that checks
the **output** finds a class of defect no amount of not-crashing would — an emitter that
quietly produces Rust rustc will reject.

### 1.1 Verified preconditions, and the one that changes the row

Measured, 2026-11-09:

```
$ which cargo-fuzz          # cargo-fuzz: absent
$ cargo +nightly --version  # nightly: absent
$ which afl                 # afl: absent
$ rustc --version
rustc 1.97.1 (8bab26f4f 2026-07-14)
```

**There is no fuzzer on this machine.** libFuzzer needs `cargo-fuzz` and a nightly toolchain,
and neither is present or pinned.

That does not make the row impossible, and it is exactly why Emerald's architecture is the one
doc 04 names: *logic in a stable crate, `fuzz_targets/*.rs` as three-line wrappers*. Here that
split stops being stylistic and becomes the whole design — §1.2.

**And one that makes the oracle cheap.** `syn` 2.0.119 is already in `Cargo.lock` as a
transitive dependency, so the generated Rust can be parsed **in process**:

```
syn = "2.0.119"   # Parser for Rust source code
```

§1.4 is why that matters more than it looks.

### 1.2 The split is load-bearing, not stylistic

Three layers:

| Layer | Where | Who runs it |
|---|---|---|
| The target logic — `fn lex_source(&[u8])`, one per target | `crates/redox-fuzz/`, stable Rust, in the workspace | `moon run :test`, every time |
| A deterministic sweep over each target | `crates/redox-fuzz/tests/` | likewise |
| The libFuzzer harnesses, three lines each | `fuzz/fuzz_targets/`, **outside** the workspace | whoever has nightly |

Without this, the row ships code nobody here can run, and a fuzz target that has never
executed is worse than none: it reads as coverage.

With it, the **logic** is exercised by the ordinary gate on every commit, and the harness is a
wrapper that adds a fuzzer when one is available. The row's value arrives immediately and grows
later, rather than arriving only on a machine nobody has.

`fuzz/` is excluded from the workspace deliberately, because `cargo-fuzz` requires it: its
crate depends on `libfuzzer-sys`, which does not build on stable, and a workspace member that
does not build breaks `cargo check` for everyone.

### 1.3 The gate's sweep is deterministic, and that is a rule

No randomness. A gate test that fuzzes with a random seed fails on one commit in fifty for a
reason nobody can reproduce, and plan 23 §10.8 is three plans' worth of evidence for how
expensive an unreproducible gate failure is.

So the sweep's inputs are fixed and derived:

1. **Every `.sdx` in the repository**, as bytes. Row 43's indentation round-trip and plan 34's
   properties already use that corpus; this adds *the same corpus, as hostile input* — the
   files are truncated, byte-flipped and spliced by a **fixed** mutation schedule.
2. **A short list of hand-written hostile cases**: empty input, a lone `\n`, invalid UTF-8,
   deep nesting, an unterminated string, an unterminated fence.

Plan 43 §1.2 corrected the dependency direction between these rows, and this is where that
lands: row 43's assertions are what row 42 points its inputs at, not the reverse. The round-trip
property and the formatter's properties become **fuzz oracles** by being handed generated input,
which is what that correction said would happen if this row arrived.

### 1.4 The real oracle: the emitter never produces invalid Rust

The fourth target is the row's centrepiece, and it is one invariant:

> If `parse_source` accepts the input, then `emit`'s output **parses as Rust**.

`syn::parse_file` answers that in microseconds, in process, with no toolchain. Shelling out to
rustc would be the stronger check and is far too slow for a fuzz target — several hundred
milliseconds against a few microseconds, which is the difference between a million inputs and a
thousand.

**What the oracle does not prove**, said plainly: `syn` accepting a file does not mean rustc
will. `syn` parses, and rustc parses *and* type-checks, borrow-checks and resolves names — so
this catches a mangled *syntax* and not a mangled *program*. That is still the defect class
that matters here, because every semantic error rustc can find is one AGENTS.md says rustc
should find; a syntax error in generated code is Redox's own bug and nobody else's.

The three corpora already compile their examples through real rustc, so the slow check exists
and is covered. This is the fast one.

---

## 2. Admission tests (doc 03 §2)

Not applicable: no construct, no lowering.

---

## 3. Concrete proof this plan targets

```bash
moon run :test                       # the deterministic sweep, in the gate
cargo fuzz run parse_program         # whoever has nightly
```

The sweep asserts, for every input in §1.3's set:

1. **No target panics.** A compiler that panics on malformed input crashes an editor — plan
   40's server runs the parser on every keystroke, so this is a product property and not a
   hygiene one.
2. **The emitter's output parses as Rust**, whenever the parser accepted the input (§1.4).
3. **Lexing is total**: `lex` returns a `Result` for every byte string and never panics,
   including on invalid UTF-8.

**Plus a negative case**, because a fuzz sweep that only asserts *no panic* over inputs that
all parse proves nothing: a hand-written input that **must** be rejected, asserted to be
rejected rather than silently accepted. The same discipline plan 43 §10.1 and plan 41's
criterion 4 record.

---

## 4. Leaves

### Leaf 1 — `leaf-target-logic`

`crates/redox-fuzz/` with the four target functions, and the deterministic sweep over §1.3's
inputs. This is the leaf that carries the value.

### Leaf 2 — `leaf-fuzz-harnesses`

`fuzz/` with `Cargo.toml`, three-line `fuzz_targets/*.rs`, a seed corpus symlinked from
`examples/`, and a README recording what it needs — nightly and `cargo-fuzz`, neither of which
is pinned.

---

## 5. Acceptance criteria

1. Four target functions, each taking `&[u8]` and returning nothing, each total.
2. The sweep runs in `moon run :test` and is **deterministic** — the same inputs in the same
   order on every run, asserted by the mutation schedule being a fixed function of an index.
3. No target panics on any input in the set.
4. The emitter's output parses as Rust for every input the parser accepted.
5. A hand-written input that must be rejected **is** rejected, so criterion 3 is not vacuous.
6. `fuzz/` is outside the workspace and `cargo check --workspace` does not try to build it.
7. The three-line harnesses are three lines: anything else belongs in the stable crate, where
   it is tested.

---

## 6. Out of scope, and what it discloses

- **Running a fuzzer.** Nothing here can: §1.1 measured it. The harnesses are for a machine
  that can, and the row's own value comes from the sweep.
- **Pinning `cargo-fuzz` and a nightly toolchain** in `devenv.nix`. That is a real option and
  deliberately declined: it doubles the toolchain this repo installs for a tool that runs on
  nobody's inner loop, and the harnesses work the moment someone adds it. Plan 41 leaf 1 pinned
  `tree-sitter` because a *committed artefact* depended on its version; nothing here does.
- **A structure-aware generator** — a function from bytes to a *valid* Redox program. It would
  find deeper defects than byte mutation, and it is most of a second parser. Named because doc
  04's row 42 mentions a "generated-program target", and what this row builds is a *mutated*
  program target: the oracle is the same and the generator is weaker. The successor is whoever
  wants `arbitrary` derived over the AST.
- **A rustc oracle.** §1.4: too slow by five orders of magnitude, and the three corpora already
  run real rustc over their examples.
- **Coverage-guided minimisation of a found crash.** `cargo fuzz tmin`'s job, on a machine with
  a fuzzer.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

`crates/emerald-fuzz-smoke`'s architecture, which doc 04 names and which §1.2 turns into the
row's design: logic in a stable crate, `fuzz_targets/*.rs` as three-line wrappers, zero
dependencies in the target crate.

**What is deliberately not taken** is its oracle. Emerald's targets assert *does not panic*,
which finds crashes and stops there. §1.4's output oracle is the row's reason to exist, and it
is available only because Redox is a *source-to-source* compiler — there is an output to check.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | Logic in a stable crate, harnesses outside the workspace | One `fuzz/` crate holding both | There is no fuzzer here (§1.1), so without the split the row ships code nobody can run — and a fuzz target that has never executed reads as coverage |
| 2 | A deterministic sweep in the gate | A random seed | An unreproducible gate failure is expensive, and plan 23 §10.8 is three plans' worth of evidence |
| 3 | `syn::parse_file` as the oracle | Shelling out to rustc | Microseconds against hundreds of milliseconds, which is the difference between a million inputs and a thousand. §1.4 states what it therefore does not prove |
| 4 | Mutate the existing corpus | Generate programs from scratch | A structure-aware generator is most of a second parser; mutation reuses thirty files that already cover every construct |
| 5 | Do not pin `cargo-fuzz` or nightly | Add both to `devenv.nix` | It doubles the installed toolchain for a tool on nobody's inner loop, and the harnesses work the moment someone adds it |
| 6 | Assert a rejection as well as the absence of panics | Only *no panic* | A sweep over inputs that all parse proves nothing about rejection, which is plan 43 §10.1's lesson and plan 41's criterion 4 |

---

## 10. Result

*Filled in when the plan ships, including anything above that turned out wrong.*
