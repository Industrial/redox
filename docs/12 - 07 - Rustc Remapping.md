# Plan 07 — Rustc Remapping

Plan ID: `rustc-remapping`
Depends on: 05 (`diagnostics-miette`), 06 (`source-map`)
Milestone: 2 — *rustc's errors land on the user's source*
Status: `done`

---

## 1. What this plan is

`voltaic check` — the subcommand that compiles a `.sdx`, runs **real rustc** over the
generated Rust, and reports every error **against the `.sdx` the user wrote**.

This is the plan doc 02 §6.5 was written for. Doc 01 §29 said "pass Rust errors through
wherever possible," which as written hands the user

```
error[E0308]: mismatched types
  --> main.xrs:47:19
```

— a file they have never opened, at a line they did not write. After this plan the same
mistake reads

```
error[E0308]: mismatched types: expected `i32`, found `&str`
  --> program.sdx:5:3
  |
5 |   print add("x", 2)
  |   ^^^^^^^^^^^^^^^^^
```

Everything about that output is rustc's except where it points, which is Voltaic's.

### 1.1 What "pass through" means precisely

The message, the code and the severity are rustc's, **verbatim**. Voltaic does not reword
`mismatched types`, does not renumber `E0308` into the `RDX` catalogue, and does not add
advice of its own. It changes exactly one thing: the position. Anything more would mean
Voltaic taking responsibility for explaining Rust's type system, which it has no business
doing and would do worse.

This is why `Diagnostic` gains a code that can be *foreign* (leaf 1) rather than
`E0308` being given an `RDX` number. A user who searches `E0308` must find Rust's
documentation, not Voltaic's.

---

## 2. Verified preconditions

Every row measured on this tree, not assumed:

| Claim | How checked | Result |
|---|---|---|
| rustc can emit one-line diagnostics | `rustc --error-format=short` on a bad file | `file:line:col: error[E0308]: mismatched types: expected \`i32\`, found \`&str\`` |
| The format is `path:line:col: level[code]: message` | same | Exactly; `error: aborting due to 1 previous error` is the only line without a position |
| The map resolves rustc's line correctly | `.tmp/typeerr.xrs:6` → map record `6 5 3 5` | source line **5**, which is the line the mistake is on |
| The column is exact on a verbatim line | `a + "x"`: rustc says `.xrs:2:7`, map shifts to `.sdx:2:5` | Both are the `+`. **Exact** |
| The column is wrong on a restructured line | `print add("x", 2)`: rustc says `.xrs:6:24`, the shift gives column 22 of a 19-character line | **Out of range** — see §2.1 |
| `VOLTAIC_RUSTC` reaches the binary, not just tests | `build.rs` uses `cargo::rustc-env` | Available via `env!` in `src/` too |

### 2.1 The precondition that failed, and what this plan does about it

Doc 02 §6.5 item 4 asks for a test asserting the remapped diagnostic reports *"that exact
`.sdx` line and column."* The line is always exact. The column is exact **only where the
emitter copies the source's content verbatim**, and the measurement above shows what
happens where it does not: `print add("x", 2)` becomes `println!("{}", add("x", 2));`, so
rustc's column 24 shifts to a column past the end of a 19-character source line.

A clamp would be the easy fix and the wrong one — it would put a caret somewhere plausible
and wrong, which in a diagnostic is worse than a caret that is honestly coarse. So:

> **Where the shifted column falls inside the origin's span, it is used. Where it does not,
> the whole span is underlined.**

`Origin` already carries the span (plan 06 leaf 1), so this costs nothing and never lies.
A reader of the second example sees the entire statement underlined, which is true: the
error is somewhere in that statement, and Voltaic does not know where.

Doc 02 §6.5 item 4 is therefore **met in two halves, measured separately** (criteria 3
and 4), rather than claimed whole. Refining the column on restructured lines needs a
per-fragment map rather than a per-line one; §6 defers it with the reason.

### 2.2 Why rustc directly, and not `cargo check`

Doc 02 §6.5 item 3 says `cargo check --message-format=json`. Two corrections:

1. **rustc, not cargo.** There is no Cargo project for the generated `.xrs` yet — cargo
   integration is doc 02 §6.6, still unresolved, and a later plan. rustc takes the same
   flags and produces the same diagnostics; cargo's stream is those diagnostics inside a
   `"compiler-message"` envelope. Plan 11 will add the envelope, not the parsing.
2. **`short`, not `json`.** The five fields this plan needs — path, line, column, code,
   message — are exactly what `--error-format=short` puts on one line. Parsing JSON would
   mean either hand-rolling a JSON parser (the kind of library function doc 02 §5.5 says
   not to write) or adding `serde_json` to read four integers and two strings. Neither
   earns its cost yet.

   What `short` loses is real and worth naming: secondary spans, notes, and machine-
   applicable suggestions. Voltaic renders one labelled span today (plan 05), so nothing is
   lost *now*. The day Voltaic wants to show rustc's suggestions, `--error-format=json` and
   `serde_json` are the upgrade, and this plan's `rustc_message` module is the only thing
   that changes.

---

## 3. Concrete proof this plan targets

Doc 02 §6.5 item 4, split per §2.1:

1. A `.sdx` with a deliberate type error on a known line, compiled by **real rustc**,
   reports **that exact `.sdx` line** — asserted with the line number written out.
2. On a verbatim line, it reports that exact `.sdx` **column** too.
3. On a restructured line, it underlines the statement's span rather than a wrong column —
   asserted, so the honest-coarseness behaviour cannot silently become a clamp.
4. On a synthetic line, it says so and reports against the `.xrs`, rather than inventing a
   source position.

Not an inspection. Four executed tests, each running the rustc the toolchain ships.

---

## 4. Leaves

### Leaf 1 — `leaf-foreign-codes`

`voltaic-diag/src/diagnostic/diagnostic_code.rs`:

```rust
pub enum DiagnosticCode {
  Voltaic(Code),       // RDX0101 — Voltaic's own catalogue
  Foreign(String),   // E0308 — rustc's, passed through untouched
}
```

`Diagnostic::code` becomes a `DiagnosticCode`. `Code` keeps its catalogue and its
completeness test; a foreign code is explicitly *not* in the catalogue, which is the point.

### Leaf 2 — `leaf-rustc-message`

`voltaic-cli/src/check_command/rustc_message.rs`. Parse one `--error-format=short` line into
a `RustcMessage { path, line, col, level, code, message }`. Lines without a position
(`error: aborting due to …`) are dropped — they are rustc's summary, not a diagnostic, and
Voltaic prints its own.

### Leaf 3 — `leaf-remap`

`voltaic-cli/src/check_command/remap.rs`. One `RustcMessage` plus the `SourceMap` becomes
either a `Diagnostic` against the `.sdx`, or — where the line is synthetic or unmapped — a
`Diagnostic` against the `.xrs` with a note saying why. The decision between a shifted
column and the whole span lives here, per §2.1.

### Leaf 4 — `leaf-check-command`

`voltaic-cli/src/check_command.rs`: `voltaic check <file.sdx> [--plain]`. Transpile in memory,
write the `.xrs` to a scratch directory, run rustc with `--emit=metadata` (no binary
needed to find type errors), remap, render, exit non-zero if anything was an error.

The `SourceMap` is held **in memory**, not read back from the sidecar: the in-memory map
carries spans and the sidecar deliberately does not (plan 06 §10.3). The sidecar is for
external tools and for plan 11.

### Leaf 5 — `leaf-rustc-at-runtime`

`voltaic-cli/src/rustc_path.rs`. Resolve rustc from `PATH` at run time, verified by its
`--version` banner, falling back to the `VOLTAIC_RUSTC` baked by `build.rs`. A shipped
binary must not depend on the rustc that happened to be present when it was built; the
baked path is the fallback, not the answer.

### Leaf 6 — `leaf-proof-and-gate`

`voltaic-cli/tests/check.rs`, four tests per §3, plus the gate and the doc updates.

---

## 5. Acceptance criteria

| # | Criterion |
|---|---|
| 1 | `voltaic check` reports a rustc error against the `.sdx`, with rustc's message and code verbatim |
| 2 | The reported **line** is the `.sdx` line the mistake is on, asserted with the number written out |
| 3 | On a verbatim line the **column** is exact too |
| 4 | On a restructured line the statement's span is underlined, not a clamped column |
| 5 | A diagnostic on a synthetic line is reported against the `.xrs`, with a note, never relocated by guesswork |
| 6 | A clean program produces no output and exits zero |
| 7 | rustc is resolved at run time and verified, not trusted from `PATH` |
| 8 | No new third-party dependency |
| 9 | Full gate green |

---

## 6. Out of scope, and what it discloses

| Deferred | To | Note |
|---|---|---|
| Column exactness on restructured lines | a later plan | Needs a per-fragment map; the per-line map cannot hold two origins for one line. §2.1 states what the user sees until then |
| rustc's secondary spans, notes and suggestions | a later plan | Needs `--error-format=json` and `serde_json`; §2.2 states the trade |
| `cargo check` and a real Cargo project | **plan 11** | Doc 02 §6.6. Same diagnostics, one envelope further out |
| Caching rustc's verdict | **plan 10** (`driver`) | `voltaic check` re-runs rustc every time |
| Warnings as anything but passed through | — | A rustc warning is reported and does not fail the check, which is rustc's own behaviour |

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format
```

---

## 8. Emerald reuse

**Lifted as a technique:** `crates/emerald-driver/build.rs` parses Cargo's NDJSON artifact
stream — the same "run the toolchain and read its machine-readable output" shape, for a
different purpose. Doc 02 §6.5 names it as the in-tree precedent, and it is: the technique
is proven, only the stream differs.

**Not applicable:** Emerald's plan 35 DWARF work. A debugger mapping machine code to `.em`
lines and a compiler mapping rustc's text to `.sdx` lines share an *aim* and no mechanism.

---

## 9. Decision log

- **rustc's message, code and level pass through verbatim; only the position changes**
  (§1.1). The alternative — renumbering `E0308` into the `RDX` catalogue — would break
  every search a user makes for a Rust error.

- **`DiagnosticCode` admits a foreign code rather than `Code` growing a variant.** A
  `Code::Rustc(String)` would put a string inside an enum whose whole value is that it is
  a closed, numbered, test-enforced catalogue (plan 05 leaf 6).

- **`--error-format=short`, not `json`** (§2.2). The fields needed fit on one line; JSON
  costs either a dependency or a hand-written parser, for structure Voltaic cannot yet
  render.

- **rustc directly, not `cargo check`** (§2.2). There is no Cargo project to check.

- **An unmapped column underlines the span rather than clamping** (§2.1). A caret that is
  plausible and wrong is worse than one that is honestly coarse.

- **A synthetic line is reported against the `.xrs`, with a note.** The alternative — the
  nearest mapped line — would be a guess presented as a fact. The note tells the user
  which file to open, which is the one thing they need.

- **rustc is resolved at run time and verified** (leaf 5). Three plans have now been bitten
  by a shim that answers to `rustc` — plan 01's `stub-ld`, plan 03's `BLAZE_SCRIPT_RUSTC`,
  plan 03's `RUSTC`-is-blaze. The rule earned by those is: never trust a name, check the
  banner.

---

## 10. Result

`done`. 233 tests pass; `fmt`, `clippy -D warnings`, `cargo audit` and the full gate are
green.

| # | Criterion | Evidence |
|---|---|---|
| 1 | rustc's message and code, verbatim, against the `.sdx` | `check.rs`: `a_type_error_is_reported_against_the_sdx` asserts `error[E0277]`, rustc's wording, and that `.xrs` appears nowhere |
| 2 | The reported line is the `.sdx` line | same test: `verbatim.sdx:2:5`, both numbers literal |
| 3 | The column is exact on a verbatim line | `the_column_is_exact_on_a_verbatim_line` — rustc's column 7 of `    a + "x"` becomes column 5 of `  a + "x"`, both the `+` |
| 4 | A restructured line underlines the statement | `a_restructured_line_underlines_the_statement` asserts the 17-caret run, so the behaviour cannot quietly become a clamp |
| 5 | A synthetic line reports against the `.xrs` | `remap.rs`: `reports_a_synthetic_line_against_the_generated_file` — unit, not end-to-end; see §10.2 |
| 6 | A clean program is silent and exits zero | `a_clean_program_passes_silently` asserts empty stderr |
| 7 | rustc resolved at run time and verified | `rustc_path.rs` + `is_real_rustc.rs`; `the_binary_resolves_rustc_itself` proves the shipped binary does it, not just the test harness |
| 8 | No new dependency | `Cargo.lock` unchanged |
| 9 | Full gate green | `moon run :check :lint :test :cargo-check :cargo-test` |

The two outputs that are the point of the plan, as `voltaic check --plain` prints them:

```text
error[E0277]: cannot add `&str` to `i32`: no implementation for `i32 + &str`
  --> tail.sdx:2:5
  |
2 |   a + "x"
  |     ^

error[E0308]: mismatched types: expected `i32`, found `&str`
  --> typeerr.sdx:5:3
  |
5 |   print add("x", 2)
  |   ^^^^^^^^^^^^^^^^^
```

Every word is rustc's. Only the position is Voltaic's — and in the first case the position is
exact to the character, while in the second it is honestly coarse.

### 10.1 What the architecture tests caught, again

Three violations, none visible while writing the code, each named precisely:

- `remap.rs` declared **four** constructs. `narrow`, `char_offset` and `generated_offset`
  became `remap/narrow.rs`, `remap/char_offset.rs`, `remap/generated_offset.rs`. Splitting
  them was not bureaucracy: `char_offset` returning `None` for an out-of-range column and
  `generated_offset` *clamping* the same case are the plan's central distinction, and in
  one file they read as an inconsistency. In two files, each carries the reason it differs.
- `rustc_path.rs` declared two. `is_real_rustc` moved to its own file, which is now where
  the three-shim history is recorded.
- `rustc_path.rs` sat at `src/` root with no children, failing §2.1. It moved under
  `check_command/`, its only caller. The rule's phrasing — *"a construct that belongs to no
  group is a signal, not an exception"* — was right: the file had no home because nothing
  but `check` needs a verified toolchain path yet.

### 10.2 Disclosures

- **Criterion 5 is proved by unit test, not end to end.** No Voltaic program available today
  makes rustc report a diagnostic on a synthetic line: the only synthetic lines are closing
  braces and blanks, and rustc attributes errors to expressions. The path is exercised by
  `remap.rs`'s tests, which is honest proof of the logic and not proof that the logic is
  reachable. When a construct arrives whose generated form rustc *does* complain about at a
  brace — a `match` with a missing arm, plausibly — that becomes an end-to-end case.

- **The column is exact only on lines the emitter copied verbatim.** Measured both ways in
  §2, asserted both ways in criteria 3 and 4. `print x` becoming `println!("{}", x)` is the
  common restructuring, and it is also the construct most likely to carry a type error — so
  this limitation is not a corner case, it is the ordinary one. The user sees the statement
  underlined rather than a wrong caret, which is the right trade, but refining it needs a
  per-fragment map.

- **`voltaic check` re-runs rustc every time.** No caching until plan 10. On these examples
  it is imperceptible; on a real program it will not be.

- **Warnings are printed and do not fail the check**, matching rustc. They are rendered to
  stderr and the exit code stays zero.

- **A rustc failure with no positioned diagnostic passes rustc's own text through.** A
  missing linker, say. Inventing a Voltaic message for a condition Voltaic does not understand
  would be worse than relaying the words of the tool that does.

### 10.3 An environment finding worth recording

The gate failed twice before passing, with blaze's daemon killed by `SIGKILL` while loading
the workspace. Nothing was wrong with the code — `cargo check`, `cargo test` (all 233),
`clippy` and `fmt` were green throughout, and `moon run :cargo-check` and `:cargo-test`
passed. The machine had 6 GB available of 61, and blaze's worker was being OOM-killed.

The cause was unrelated to Voltaic: a `git commit` in another repository had entered an
infinite pre-commit recursion and accumulated 1,296 processes holding 37 GB. With that
cleared the identical gate passed unchanged.

The lesson for this repo is the one AGENTS.md already half-records for stale blaze state:
**a blaze failure that names crates you did not touch is usually not about your code.**
`SIGKILL` during *"load workspace"* specifically means the daemon was killed rather than
having rejected anything — check `free -h` before reading the diff. AGENTS.md gains this.
