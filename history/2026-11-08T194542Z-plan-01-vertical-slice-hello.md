2026-11-08T19:45:42Z

---
name: Vertical Slice — `.sdx` to a Running Binary
overview: "The smallest thing that proves the whole pipeline: `main = -> print \"hello\"` becomes a `.xrs` file containing ordinary Rust, which real rustc compiles into a binary that prints `hello`. Nothing about the language is interesting at this size and that is the point — every later plan extends this skeleton rather than restructuring it. Three things are deliberately built now rather than later because doc 02 and Emerald's own history both show them to be expensive retrofits: byte spans on every AST node, comments as trivia attached to AST nodes, and whitespace-adjacency information on every token. The parser-strategy bake-off doc 02 §3.5 requires is a leaf here rather than a separate plan, because both candidate parsers have to parse this same slice anyway."
maestro:
  mission_id: null
  spec_path: null
  execution_overlay: null
todos:
  - id: leaf-workspace-scaffold
    content: "Create the eight-crate workspace from doc 02 §5.4 — `redox-lexer`, `redox-ast`, `redox-parser`, `redox-emit`, `redox-diag`, `redox-driver`, `redox-fmt`, `redox-cli` — with only the first four, plus `redox-cli`, carrying code in this plan; the rest are empty crates with a module doc comment naming the plan that fills them. Use `[workspace.package]` version/edition/license inheritance as `/data/Code/emerald/Cargo.toml` does, so per-crate manifests stay three lines. Replace the existing `crates/app` stub. Add the CI line-cap check doc 02 §5.4 calls for (no single `.rs` over ~1,500 lines) — Emerald's `emerald-codegen/src/lib.rs` is 34,429 lines in one file and that is the one clear structural defect in an otherwise well-organised repository; the check costs nothing now and is unenforceable later."
    status: pending
  - id: leaf-lexer-minimal-offside
    content: "`redox-lexer`, hand-written, covering only what this slice needs: identifiers, string literals, `=`, `->`, newline, and INDENT/DEDENT from a four-space off-side rule with tabs rejected outright (doc 01 §3.1 permits either rejection or one documented normalisation; rejection is one line of code and zero ambiguity). Three properties are mandatory now and must not be deferred: (1) every token carries a byte span; (2) comments and blank-line runs are emitted as `Trivia` attached to the following token, never discarded; (3) every token records whether whitespace preceded it. Items (1) and (2) are the retrofit hazards — see this plan's Decision log. Item (3) is free here and is what plan 19's call grammar and plan 27's `(-1)`-versus-`(- 1)` rule both depend on, per doc 03 §5.1 and §5.4."
    status: pending
  - id: leaf-ast-with-spans-and-trivia
    content: "`redox-ast`: types only, no logic. Every node carries `span: (usize, usize)` and `leading: Vec<Trivia>` plus `trailing: Option<Trivia>`. Deliberately not generic over a span type and deliberately not an arena — doc 01 §30's minimality constraint, and neither is needed until a profile says so. Node set for this plan: `Program`, `Item::Fn`, `Block`, `Stmt::Expr`, `Expr::Call`, `Expr::StrLit`, `Expr::Ident`."
    status: pending
  - id: leaf-parser-bakeoff
    content: "Build this slice's parser twice — hand-written recursive descent, and LALRPOP — and publish the comparison in `docs/COMPILER.md` with a named revisit trigger, following `/data/Code/emerald/spec/COMPILER.md`'s format exactly (a table of axes, the decision, the rationale, and the evidence paths). Doc 02 §5.2 recommends hand-written on five arguments and doc 03 §23 item 6 records that the recommendation stands on argument rather than measurement; this leaf is that measurement. Axes to compare, fixed before building so the result cannot be rationalised: can it express the off-side rule without lexer/parser feedback; can it preserve trivia; does it recover from more than one error in a pass; what is the error message for a missing `->`; what is the setup cost; and how many lines is the slice. Record the result even if it contradicts doc 02 — Emerald tuned Cranelift *before* its bake-off specifically so the comparison could not be rigged, and that is the standard to meet."
    status: pending
  - id: leaf-emitter-and-xrs
    content: "`redox-emit`: `Program` to Rust source text, deterministic and byte-stable across runs, with trivia re-emitted (`#` to `//`). Write `main.xrs`. Do not shell out to rustfmt as part of the contract — doc 02 §5.3's reasoning is that it would make output depend on an external tool's version, break determinism, and destroy the span map; offer it later as an opt-in flag that marks the map invalid. Emit one source line per input line wherever the construct permits it, which at this size is everywhere: this is doc 02 §6.5 item 1's line-preserving discipline, and establishing it now is what keeps plan 06's span map cheap."
    status: pending
  - id: leaf-cli-build
    content: "`redox-cli` with exactly one working subcommand, `redox build <file.sdx> [-o <out.xrs>]`, plus `--version` and `--help`. Default output mapping `main.sdx` to `main.xrs` per doc 01 §3.4. No `check`, no `watch`, no `fmt` — those are plans 11 and 12, and doc 01 §27 says not to add unnecessary commands. The binary is named `redox`; the crate stays `redox-cli`, matching Emerald's `[[bin]] name = \"emerald\"` convention for the same reason (`cargo build -p redox-cli` keeps working)."
    status: pending
  - id: leaf-test-harness
    content: "Adapt `/data/Code/emerald/crates/emerald-cli/tests/hello_em.rs` into Redox's end-to-end harness: locate the example via `CARGO_MANIFEST_DIR`, invoke the real built binary via `env!(\"CARGO_BIN_EXE_redox\")`, then — the step Emerald does not need — invoke real `rustc` on the generated `.xrs` and run the resulting binary, asserting exact stdout. Add `examples/hello.sdx` as the first entry in what becomes the CI-checked source of truth (doc 02 §8.3). Hard convention, established here and never broken: **every `.sdx` fixture is a file and tests reference files** — Emerald's plan 71 cutover missed 113 fixtures embedded as string literals inside Rust test modules, and that is a cost paid once per syntax change forever."
    status: pending
  - id: leaf-golden-harness
    content: "`tests/golden/`: `hello.sdx` beside its expected `hello.xrs`, compared as exact text, with a `--bless` path to regenerate. Doc 01 §3.4 requires deterministic readable output, which is what makes exact-text golden tests possible at all. Golden text proves stability and the compile-and-run test above proves correctness; doc 02 §8.1–8.2 is explicit that neither substitutes for the other, so both exist from the first plan."
    status: pending
  - id: leaf-docs-and-gate
    content: "Author `docs/COMPILER.md` (the bake-off result and its revisit trigger) and update `AGENTS.md`'s build/test section for the new workspace layout. Run the full gate from `docs/04 - Redox Plan of Plans.md`: `moon run :check :lint :test :cargo-check :cargo-test :format`. Independently re-verify beyond any single agent's self-report — at minimum, run `redox build examples/hello.sdx`, compile the output with rustc by hand, execute it, and confirm the stdout matches. Emerald's plans 71, 73 and 74 each record an independent re-check beyond the implementing agent's report, and the AGENTS.md tooling-gotchas section documents two environment traps that each produced a false compiler-bug report before the cause was found."
    status: pending
isProject: false
---

# Plan 01 — Vertical Slice

## What this plan is

`docs/01 - Redox.md` §31 Phase 1 lists ten things to implement and gives this acceptance
test:

```
add = (a: i32, b: i32): i32 ->
  a + b

main = ->
  print add 10, 20
```

This plan is deliberately **smaller than that**. It implements:

```
main = ->
  print "hello"
```

and nothing else. The §31 Phase 1 test — which additionally needs parameters, type
annotations, return types, binary operators, and a nested parenless call — is plan 03's
acceptance test, after plan 02 has finished the lexer and plan 19 has settled the call
grammar.

The reason for going smaller is that this slice is the minimum that exercises **every stage
of the pipeline end to end**: file read, off-side lexing, parsing, Rust emission, file write,
and then real rustc producing a real binary that produces real output. Nothing in it is
throwaway. Every later plan adds productions to a parser that exists, lowerings to an emitter
that exists, and examples to a harness that exists.

## Concrete proof this plan targets

`examples/hello.sdx`:

```
# The smallest Redox program.
main = ->
  print "hello"
```

`redox build examples/hello.sdx` produces `examples/hello.xrs`:

```rust
// The smallest Redox program.
fn main() {
    println!("hello");
}
```

and the executable test asserts, in one run: the transpile exits zero; the `.xrs` matches
the golden file byte for byte; `rustc` compiles it without warnings; the binary exits zero;
and its stdout is exactly `hello\n`.

Note what the proof includes that a transpiler is tempted to skip. Asserting the generated
text is not sufficient — text can be stable and wrong. Asserting that rustc accepts it is
not sufficient either — it can compile and do the wrong thing. Both plus execution is the
cheapest complete proof, and doc 02 §8.1–8.2 treats the distinction as load-bearing.

## Decision log

- **Spans, trivia and adjacency are built now, not later.** This is the single most
  consequential decision in the plan, and it is made on evidence rather than taste. Emerald's
  plan 22 (`sema-diagnostic-spans`) discovered that `TypeExpr` carried no span at all, so
  every `resolve_type` diagnostic hardcoded `(0, 0)` and an illegal type on line 6 reported
  line 1, column 1. That was found **twenty-one plans after** the diagnostics plan shipped,
  and the fix was then "judged too large/risky" relative to its value, so it shipped a
  parameter-threading workaround with a disclosed residual gap at call sites that have no
  real span in scope. Separately, `emerald-fmt`'s own module doc discloses that every
  non-doc comment is lost, because the lexer treats comments as whitespace and they have no
  AST representation — for a formatter that is an annoyance; for Redox, where comments must
  reach the generated `.xrs`, it would be a product defect. Both are cheap now and
  structurally expensive later, so both are leaves of plan 01.

- **The parser bake-off is a leaf here, not a separate plan.** Emerald made it plan 02,
  before the workspace bootstrap, and that ordering is right in principle. But both
  candidate parsers must parse this same slice to be compared at all, so splitting them into
  two plans would mean building the slice twice. The bake-off's integrity is preserved by
  fixing the comparison axes *before* building either parser, which is the same safeguard
  Emerald applied when it tuned Cranelift's optimisation level before measuring it against
  LLVM so the result could not be rigged.

- **The emitter does not call rustfmt.** Doc 02 §5.3's three reasons stand: output would
  depend on an external tool's version, determinism would break, and the span map plan 06
  needs would be destroyed. An opt-in flag can exist later, marked as invalidating the map.

- **Eight crates, five of them empty.** Doc 02 §5.4's split is adopted whole at the start
  rather than grown, because crate boundaries are the one thing that is genuinely painful to
  introduce late, and because empty crates carrying a module doc comment that names their
  owning plan are self-documenting. The counter-argument — that this is scaffolding before
  it is needed — is real but small: five three-line manifests.

- **`print` stays a special case for exactly one plan.** `print "hello"` lowers to
  `println!("hello")` per doc 01 §3.3. Doc 03 §10.9 establishes that this is the *only*
  sanctioned sugar of its kind and that it is also exactly Emerald's `puts` situation — a
  hardcoded grammar keyword that papered over a general parenless-call ambiguity for 89
  plans. Plan 19 resolves the general case and this special case then dissolves. Recorded
  here so the shortcut is a dated loan rather than a permanent feature.

- **Tabs are rejected, not normalised.** Doc 01 §3.1 permits either. Rejection is one branch
  and removes a class of indentation bug that is otherwise reported against the wrong line.

## Emerald reuse

Stated precisely, because doc 02 §2 found that most of Emerald does not transfer and a plan
claiming otherwise would be overstating it.

**Lifted, with adaptation:**

| From | To | What |
|---|---|---|
| `crates/emerald-cli/tests/hello_em.rs` | `crates/redox-cli/tests/hello.rs` | The end-to-end harness shape: `CARGO_MANIFEST_DIR` to locate the fixture, `env!("CARGO_BIN_EXE_…")` to invoke the genuinely-built binary, assert exact stdout, clean up. Redox adds an intermediate rustc invocation Emerald has no need for. |
| `Cargo.toml` | `Cargo.toml` | `[workspace.package]` inheritance of version, edition, license and repository, so per-crate manifests stay three lines. |
| `crates/emerald-cli/src/main.rs` | `crates/redox-cli/src/main.rs` | The `[[bin]]`-name-differs-from-crate-name convention, for the same reason: `cargo build -p redox-cli` keeps working while the shipped binary is `redox`. |

**Consulted as methodology, no code:** `spec/COMPILER.md`'s decision-record format for the
bake-off leaf — a table of axes, the decision, the rationale, the evidence paths, and a
named revisit trigger.

**Deliberately not taken:**

| Not taken | Why |
|---|---|
| `crates/emerald-lexer` (116 lines, `logos`) | Regex-dispatch lexing cannot express an off-side rule, and Redox additionally needs trivia and whitespace-adjacency on every token. |
| `crates/emerald-parser` (LALRPOP) | The `leaf-parser-bakeoff` leaf exists to decide this on measurement, but doc 02 §5.2's prior is that LALR(1) is the wrong tool: it is what blocked Emerald's own parenless calls for 89 plans, it fits indentation-sensitive grammars badly, and it made both error recovery and trivia preservation retrofits. |
| `crates/emerald-codegen` (34,429 lines, LLVM) | Redox emits Rust source text. No analogue. |
| `crates/emerald-sema` (22,589 lines) | Redox has rustc. Doc 02 §6.2's rule: every check rustc can make, rustc makes. |
| `crates/emerald-fmt` | Its approach is the documented anti-pattern — a pure AST pretty-printer that loses every comment. Plan 34 builds a trivia-preserving formatter instead. |
| `crates/emerald-rt` (~58k lines) | Doc 02 §1.1: 112 of Emerald's 203 plans built a standard library. Redox has crates.io. No Redox plan ever implements a library function. |

## Out of scope, and what it discloses

Each item below is the acceptance criterion of a named successor, so nothing here is left
without a venue:

- **Parameters, types, return types, operators, nested calls** — plan 03, which takes doc 01
  §31 Phase 1's own acceptance test as its proof.
- **The rest of the off-side rule** — continuation suppression inside delimiters and after
  trailing operators, the `#` longest-match table, interpolation, `"""` — plan 02. This slice
  needs none of it and guessing at it now would be designing against doc 03 §4.1 from memory.
- **`miette` diagnostics** — plan 05. This plan's errors may be plain one-line messages with
  a file, line and column; they must carry spans, which the AST already does.
- **The span map and rustc remapping** — plans 06 and 07. This plan establishes
  line-preserving emission, which is what makes them cheap, but writes no map.
- **Cargo integration** — plan 09. This plan writes a `.xrs` and the test invokes rustc
  directly; no `build.rs`, no `OUT_DIR`, no `include!`.
- **The general parenless-call grammar** — plan 19. `print` is hardcoded until then, on the
  dated loan recorded in the Decision log.

## Acceptance criteria

1. `redox build examples/hello.sdx` exits zero and writes `examples/hello.xrs`.
2. The generated text matches `tests/golden/hello.xrs` byte for byte.
3. `rustc` compiles the generated file with no errors and no warnings.
4. The resulting binary exits zero and prints exactly `hello\n`.
5. The comment in `hello.sdx` appears in the generated Rust as `//`.
6. Every AST node carries a byte span, verified by a unit test asserting a known node's
   span against hand-computed offsets.
7. Two consecutive `redox build` runs over the same input produce byte-identical output.
8. A tab in the indentation of a `.sdx` file is rejected with a diagnostic naming the line.
9. `docs/COMPILER.md` records the bake-off with its axes, decision, rationale, evidence
   paths and revisit trigger.
10. No single `.rs` file in the workspace exceeds the CI line cap.
11. The full gate is green: `moon run :check :lint :test :cargo-check :cargo-test :format`.
