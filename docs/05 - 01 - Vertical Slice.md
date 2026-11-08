# Plan 01 — Vertical Slice

Plan: 01 of 48 (`docs/04 - Redox Plan of Plans.md`, Milestone 1)
Slug: `vertical-slice-hello`
Status: **`done`** — shipped 2026-11-08; see §10 for the evidence per criterion
Depends on: nothing
Discloses: plans 02, 03, 05, 06, 07, 09, 19

---

## 1. What this plan is

`docs/01 - Redox.md` §31 Phase 1 lists ten things to implement and gives this acceptance
test:

```
add = (a: i32, b: i32): i32 ->
  a + b

main = ->
  print add 10, 20
```

**This plan is deliberately smaller.** It implements:

```
main = ->
  print "hello"
```

and nothing else. §31's test additionally needs parameters, type annotations, return types,
binary operators and a nested parenless call; it becomes **plan 03's** acceptance test,
after plan 02 finishes the lexer.

The reason for going smaller is that this slice is the minimum that exercises **every stage
of the pipeline end to end** — file read, off-side lexing, parsing, Rust emission, file
write, then real rustc producing a real binary that produces real output. Nothing in it is
throwaway. Every later plan adds productions to a parser that exists, lowerings to an emitter
that exists, and fixtures to a harness that exists.

Three things are built now rather than deferred, on evidence rather than taste: **byte spans
on every AST node**, **comments as trivia attached to AST nodes**, and **whitespace-adjacency
on every token**. §9 gives the evidence.

---

## 2. Verified precondition: rustc accepts `.xrs` directly

`docs/01 - Redox.md` §1.1 raises this as an open question:

> The compiler must account for Rust compiler input-file extension handling. If necessary,
> it should use standard `.rs` staging files or compiler input through stdin rather than
> requiring changes to Rust itself.

**Measured, not assumed** (2026-11-08, rustc 1.97.1, this repository's devenv shell):

```
$ printf '// probe\nfn main() {\n    println!("hello");\n}\n' > hello.xrs
$ rustc --edition 2024 -o out hello.xrs
RUSTC_EXIT=0        stderr: 0 bytes
$ ./out
hello               EXIT=0
```

**rustc compiles a `.xrs` file directly, with zero warnings, and the binary runs.** No `.rs`
staging file and no stdin plumbing is required for the direct-rustc path, so §1.1's "if
necessary" clause does not fire and this plan's test harness invokes rustc on the `.xrs`
as-is.

**Scope limit, stated so plan 09 is not surprised.** This result is about `rustc` invoked
directly. **Cargo is different**: it discovers crate roots by name (`src/main.rs`,
`src/lib.rs`) and will not pick up a `.xrs`. That is precisely why doc 02 §6.6 recommends
the `build.rs` + `OUT_DIR` + `include!` route, and this finding neither supports nor
weakens that recommendation — it only removes a worry from *this* plan's harness.

---

## 3. Concrete proof this plan targets

**`examples/hello.sdx`** — the first entry in what becomes the CI-checked source of truth
(doc 02 §8.3):

```
# The smallest Redox program.
main = ->
  print "hello"
```

**`redox build examples/hello.sdx`** produces **`examples/hello.xrs`**:

```rust
// The smallest Redox program.
fn main() {
    println!("hello");
}
```

The executable test asserts, in one run:

1. the transpile exits zero;
2. the generated text matches `tests/golden/hello.xrs` byte for byte;
3. `rustc --edition 2024` compiles it with **zero stderr output**;
4. the binary exits zero;
5. its stdout is exactly `hello\n`.

Note what this includes that a transpiler is tempted to skip. Asserting the generated text
is not sufficient — text can be stable and wrong. Asserting that rustc accepts it is not
sufficient either — it can compile and do the wrong thing. Text plus compilation plus
execution is the cheapest complete proof, and doc 02 §8.1–8.2 treats the distinction as
load-bearing.

---

## 4. Leaves

### Leaf 1 — `leaf-workspace-scaffold`

Create doc 02 §5.4's eight-crate workspace, replacing the existing `crates/app` stub.

```
crates/
  redox-lexer/    code in this plan
  redox-ast/      code in this plan
  redox-parser/   code in this plan
  redox-emit/     code in this plan
  redox-cli/      code in this plan
  redox-diag/     empty; module doc names plan 05
  redox-driver/   empty; module doc names plan 10
  redox-fmt/      empty; module doc names plan 34
```

- Use `[workspace.package]` inheritance for version, edition, license and repository, as
  `/data/Code/emerald/Cargo.toml` does, so per-crate manifests stay three lines.
- The shipped binary is `redox`; the crate stays `redox-cli` (`[[bin]] name = "redox"`),
  matching Emerald's convention so `cargo build -p redox-cli` keeps working.
- Empty crates carry a `//!` module doc naming the plan that fills them. They are
  self-documenting and cost five three-line manifests.
- **Add the CI line-cap check** doc 02 §5.4 calls for: no single `.rs` file over ~1,500
  lines. Emerald's `crates/emerald-codegen/src/lib.rs` is **34,429 lines in one file** and
  `emerald-sema/src/lib.rs` is 22,589 — the one clear structural defect in an otherwise
  well-organised repository. The check costs nothing now and is unenforceable later.
- Edition 2024; MSRV 1.88 per doc 03 §19, though nothing in this slice needs it.

### Leaf 2 — `leaf-lexer-minimal-offside`

`redox-lexer`, hand-written. Token set for this slice only:

| Token | Text |
|---|---|
| `Ident` | `main`, `print` |
| `StrLit` | `"hello"` (no interpolation yet — plan 23) |
| `Eq` | `=` |
| `Arrow` | `->` |
| `Newline` | logical line break |
| `Indent` / `Dedent` | off-side rule |
| `Eof` | |

**Off-side algorithm** (the full rule, including continuation suppression, is plan 02):

```
indent_stack = [0]
at the start of each logical line:
    if the line is blank or comment-only:
        accumulate as Trivia; do not touch indent_stack
    measure leading spaces; a tab is an error (see below)
    if width > top:    push(width);  emit Indent
    elif width == top:               emit Newline
    else:
        while width < top: pop(); emit Dedent
        if width != top: error — inconsistent dedent
at EOF: emit Dedent for every level above 0
```

Four spaces per level. **A tab in leading whitespace is rejected** with a diagnostic naming
the line. Doc 01 §3.1 permits either rejection or one documented normalisation; rejection is
one branch and removes a class of indentation bug that otherwise reports against the wrong
line.

**Three mandatory properties.** None may be deferred:

1. **Every token carries a byte span** `(start, end)`.
2. **Comments and blank-line runs are emitted as `Trivia` attached to the following
   token**, never discarded.
3. **Every token records whether whitespace preceded it** (`ws_before: bool`).

(1) and (2) are the retrofit hazards — see §9. (3) is free here, and is what plan 19's call
grammar and plan 27's `(-1)`-versus-`(- 1)` rule both depend on (doc 03 §5.1, §5.4).

### Leaf 3 — `leaf-ast-with-spans-and-trivia`

`redox-ast`: types only, no logic.

```rust
pub type Span = (usize, usize);

pub enum TriviaKind { LineComment, BlankLines }
pub struct Trivia { pub kind: TriviaKind, pub text: String, pub span: Span }

pub struct Program { pub items: Vec<Item>, pub span: Span }
pub enum   Item    { Fn(FnDecl) }
pub struct FnDecl  { pub name: Ident, pub body: Block, pub span: Span,
                     pub leading: Vec<Trivia>, pub trailing: Option<Trivia> }
pub struct Block   { pub stmts: Vec<Stmt>, pub span: Span }
pub enum   Stmt    { Expr(Expr) }
pub enum   Expr    { Call { callee: Box<Expr>, args: Vec<Expr>, span: Span },
                     StrLit { value: String, span: Span },
                     Ident(Ident) }
pub struct Ident   { pub text: String, pub span: Span }
```

Every node carries a span; every item-level node carries trivia. Deliberately **not**
generic over a span type and deliberately **not** arena-allocated — doc 01 §30's minimality
constraint, and neither is needed until a profile says otherwise.

### Leaf 4 — `leaf-parser-bakeoff`

Build this slice's parser **twice** — hand-written recursive descent, and LALRPOP — and
publish the comparison in `docs/COMPILER.md`.

Doc 02 §5.2 recommends hand-written on five arguments; doc 03 §23 item 6 records that the
recommendation **stands on argument rather than measurement**. This leaf is that
measurement.

**Axes fixed before either parser is built**, so the result cannot be rationalised after the
fact:

| # | Axis |
|---|---|
| 1 | Can it express the off-side rule without lexer/parser feedback? |
| 2 | Can it preserve trivia onto AST nodes? |
| 3 | Does one pass recover from more than one error? |
| 4 | What is the diagnostic for a missing `->`? |
| 5 | Setup cost (build script, generated code, build time) |
| 6 | Lines of code for this slice |

Record the result **even if it contradicts doc 02**. Emerald tuned Cranelift's optimisation
level *before* measuring it against LLVM specifically so the bake-off could not be rigged
(`/data/Code/emerald/spec/COMPILER.md`), and that is the standard to meet. Follow that
file's format exactly: a table of axes, the decision, the rationale, the evidence paths, and
a **named revisit trigger**.

**Grammar for this slice**, whichever parser wins:

```
Program  := Item* Eof
Item     := Ident "=" "->" Newline Indent Stmt+ Dedent
Stmt     := Expr Newline
Expr     := Ident StrLit        # parenless call, one argument, statement-initial only
          | StrLit
          | Ident
```

The one-argument statement-initial restriction is the minimum this slice needs. **Plan 19**
generalises it to doc 03 §5.1's full call grammar; this is Emerald's plan-07 restriction
applied knowingly rather than inherited by accident.

### Leaf 5 — `leaf-emitter-and-xrs`

`redox-emit`: `Program` → Rust source text, written to `.xrs`.

| Redox | Rust |
|---|---|
| `main = ->` + block | `fn main() { … }` |
| `print <StrLit>` | `println!(<StrLit>);` |
| `# text` trivia | `// text` |
| blank-line run | preserved, collapsed to at most one |

- **Deterministic and byte-stable across runs.** No `HashMap` iteration anywhere in the
  emitter.
- **No rustfmt in the contract.** Doc 02 §5.3: it would make output depend on an external
  tool's version, break determinism, and destroy the span map plan 06 needs. An opt-in flag
  can exist later, marked as invalidating the map.
- **One output line per input line wherever the construct permits**, which at this size is
  everywhere. This is doc 02 §6.5 item 1's line-preserving discipline; establishing it now
  is what keeps plan 06's span map cheap.
- Four-space indentation in the output, matching rustfmt's default so generated code does
  not read as foreign.

### Leaf 6 — `leaf-cli-build`

`redox-cli`, with exactly one working subcommand:

```
redox build <file.sdx> [-o <out.xrs>]
redox --version
redox --help
```

Default output mapping `main.sdx` → `main.xrs` (doc 01 §3.4). **No `check`, no `watch`, no
`fmt`** — plans 11 and 12, and doc 01 §27 says not to add unnecessary commands. Diagnostics
at this stage may be plain one-line messages carrying file, line and column; `miette`
rendering is plan 05.

### Leaf 7 — `leaf-test-harness`

Adapt `/data/Code/emerald/crates/emerald-cli/tests/hello_em.rs` into
`crates/redox-cli/tests/hello.rs`:

1. locate the fixture via `CARGO_MANIFEST_DIR`;
2. invoke the genuinely-built binary via `env!("CARGO_BIN_EXE_redox")`;
3. **then the step Emerald has no need for** — invoke real `rustc --edition 2024` on the
   generated `.xrs` (§2 proves this works), assert zero stderr;
4. run the resulting binary, assert exit zero and exact stdout;
5. clean up.

**Hard convention, established here and never broken: every `.sdx` fixture is a file, and
tests reference files.** Emerald's plan 71 cutover missed **113 fixtures** embedded as
string literals inside `emerald-codegen`, `emerald-driver`, `emerald-cli` and `emerald-mcp`
test modules. That is a cost paid once per syntax change, forever.

### Leaf 8 — `leaf-golden-harness`

`tests/golden/hello.sdx` beside its expected `hello.xrs`, compared as exact text, with a
`--bless` path to regenerate. Doc 01 §3.4's determinism requirement is what makes exact-text
golden tests possible at all.

Golden text proves **stability**; leaf 7's compile-and-run proves **correctness**. Doc 02
§8.1–8.2 is explicit that neither substitutes for the other, so both exist from the first
plan.

### Leaf 9 — `leaf-docs-and-gate`

- Author **`docs/COMPILER.md`**: the bake-off result, its axes, the decision, the rationale,
  the evidence paths and the revisit trigger. Also record §2's rustc-extension finding, so
  it is not re-derived.
- Update `AGENTS.md`'s build/test section for the new workspace layout.
- Run the full gate (§7).
- **Independently re-verify beyond any single agent's self-report**: run
  `redox build examples/hello.sdx`, compile the output with rustc by hand, execute it, and
  confirm the stdout. Emerald's plans 71, 73 and 74 each record an independent re-check
  beyond the implementing agent's report, and its `AGENTS.md` documents two environment
  traps that each produced a **false compiler-bug report** before the cause was found.

---

## 5. Acceptance criteria

| # | Criterion |
|---|---|
| 1 | `redox build examples/hello.sdx` exits zero and writes `examples/hello.xrs` |
| 2 | Generated text matches `tests/golden/hello.xrs` byte for byte |
| 3 | `rustc --edition 2024` compiles the generated file with **zero stderr output** |
| 4 | The resulting binary exits zero and prints exactly `hello\n` |
| 5 | The comment in `hello.sdx` appears in the generated Rust as `//` |
| 6 | Every AST node carries a byte span, verified by a unit test asserting a known node's span against hand-computed offsets |
| 7 | Two consecutive `redox build` runs over the same input produce byte-identical output |
| 8 | A tab in leading whitespace is rejected with a diagnostic naming the line |
| 9 | An inconsistent dedent is rejected with a diagnostic naming the line |
| 10 | `docs/COMPILER.md` records the bake-off with axes, decision, rationale, evidence paths and revisit trigger |
| 11 | No single `.rs` file in the workspace exceeds the CI line cap |
| 12 | The full gate is green |

---

## 6. Out of scope, and what it discloses

Each item is the acceptance criterion of a named successor, so nothing is left without a
venue:

| Deferred | To | Note |
|---|---|---|
| Parameters, types, return types, operators, nested calls | **plan 03** | Takes doc 01 §31 Phase 1's own test as its proof |
| Continuation suppression, `#` longest-match table, interpolation, `"""` | **plan 02** | This slice needs none of it; guessing now would mean designing against doc 03 §4.1 from memory |
| `miette` diagnostics, `RDX` code catalogue | **plan 05** | Errors here may be plain messages; they must carry spans, which the AST already does |
| Span map, rustc diagnostic remapping | **plans 06, 07** | This plan establishes line-preserving emission, which makes them cheap, but writes no map |
| Cargo integration | **plan 09** | §2's scope limit is the relevant finding |
| General parenless-call grammar | **plan 19** | `print` is hardcoded until then — see §9 |

---

## 7. Gate

```bash
moon run :check          # blaze check — fast inner-loop verdict
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # unit, integration, doctests
moon run :cargo-check    # rustc's verdict — AGENTS.md: rustc is the final word
moon run :cargo-test     # cargo nextest run
moon run :format         # treefmt
```

Plus, from this plan onward: every example `.sdx` transpiles, its generated Rust compiles
under real rustc, runs, and its stdout is asserted.

---

## 8. Emerald reuse

Stated precisely, because doc 02 §2 found that most of Emerald does not transfer and a plan
claiming otherwise would be overstating it.

**Lifted, with adaptation:**

| From | To | What |
|---|---|---|
| `crates/emerald-cli/tests/hello_em.rs` | `crates/redox-cli/tests/hello.rs` | The harness shape: `CARGO_MANIFEST_DIR` to locate the fixture, `env!("CARGO_BIN_EXE_…")` to invoke the genuinely-built binary, assert exact stdout, clean up. Redox inserts a rustc step Emerald has no need for. |
| `Cargo.toml` | `Cargo.toml` | `[workspace.package]` inheritance of version, edition, license, repository. |
| `crates/emerald-cli/Cargo.toml` | `crates/redox-cli/Cargo.toml` | The `[[bin]]`-name-differs-from-crate-name convention, for the same stated reason. |

**Consulted as methodology, no code:** `spec/COMPILER.md`'s decision-record format for
leaf 4, and its tune-the-incumbent-first safeguard.

**Deliberately not taken:**

| Not taken | Why |
|---|---|
| `crates/emerald-lexer` (116 lines, `logos`) | Regex-dispatch lexing cannot express an off-side rule, and Redox additionally needs trivia and adjacency on every token. |
| `crates/emerald-parser` (LALRPOP) | Leaf 4 decides this on measurement, but doc 02 §5.2's prior is that LALR(1) is the wrong tool: it is what blocked Emerald's own parenless calls for 89 plans, fits indentation-sensitive grammars badly, and made both error recovery and trivia preservation retrofits. |
| `crates/emerald-codegen` (34,429 lines, LLVM) | Redox emits Rust source text. No analogue. |
| `crates/emerald-sema` (22,589 lines) | Redox has rustc. Doc 02 §6.2: every check rustc can make, rustc makes. |
| `crates/emerald-fmt` | Its approach is the documented anti-pattern — a pure AST pretty-printer that loses every comment. Plan 34 builds a trivia-preserving formatter instead. |
| `crates/emerald-rt` (~58k lines) | Doc 02 §1.1: 112 of Emerald's 203 plans built a standard library. Redox has crates.io. **No Redox plan ever implements a library function.** |

---

## 9. Decision log

- **Spans, trivia and adjacency are built now, not later.** The most consequential decision
  in the plan, made on evidence. Emerald's plan 22 (`sema-diagnostic-spans`) discovered that
  `TypeExpr` carried **no span at all**, so every `resolve_type` diagnostic hardcoded
  `(0, 0)` and an illegal type on line 6 reported line 1, column 1. That was found
  **twenty-one plans after** the diagnostics plan shipped, and the full fix was by then
  "judged too large/risky" relative to its value — so it shipped a parameter-threading
  workaround with a disclosed residual gap at call sites that have no real span in scope.
  Separately, `emerald-fmt`'s own module doc discloses that **every non-doc comment is
  lost**, because its lexer treats comments as whitespace and they have no AST
  representation. For a formatter that is an annoyance; for Redox, where comments must reach
  the generated `.xrs` (doc 02 §10.1), it would be a product defect. Both are cheap now and
  structurally expensive later.

- **The parser bake-off is a leaf here, not a separate plan.** Emerald made it plan 02,
  before its workspace bootstrap, and that ordering is right in principle. But both
  candidate parsers must parse this same slice to be compared at all, so splitting them
  would mean building the slice twice. The bake-off's integrity is preserved instead by
  fixing the axes before either parser exists.

- **rustc's `.xrs` handling was measured, not assumed** (§2). Doc 01 §1.1 left it open with
  a conditional workaround; the condition does not hold for direct rustc invocation. One
  probe removed a contingency from this plan's harness.

- **The emitter does not call rustfmt.** Doc 02 §5.3's three reasons stand: external version
  dependence, broken determinism, destroyed span map.

- **Eight crates, five of them empty.** Crate boundaries are the one thing genuinely painful
  to introduce late. The counter-argument — scaffolding before it is needed — is real but
  small: five three-line manifests.

- **`print` stays a special case for exactly one plan.** `print "hello"` → `println!(…)` per
  doc 01 §3.3. Doc 03 §10.9 establishes that this is the only sanctioned sugar of its kind
  *and* that it is exactly Emerald's `puts` situation — a hardcoded grammar keyword that
  papered over a general parenless-call ambiguity for 89 plans
  (`grammar.lalrpop:622-626`). Plan 19 resolves the general case and this special case then
  dissolves. Recorded here so the shortcut is a **dated loan**, not a permanent feature.

- **Tabs are rejected, not normalised.** Doc 01 §3.1 permits either; rejection is one branch
  and removes a class of misreported bug.

- **`main` is not yet special.** The emitter maps any `name = ->` item to `fn name() { … }`;
  `main` is simply the one this fixture uses. No entry-point handling, no `#[no_main]`
  concerns — plan 09's business.

---

## 10. Shipped — evidence per criterion

Executed 2026-11-08. Full gate green: `moon run :check :lint :test :cargo-check :cargo-test
:ci-format` all exit 0. **50 tests pass**, zero clippy warnings, treefmt reports zero files
changed.

| # | Criterion | Evidence |
|---|---|---|
| 1 | `redox build` exits zero and writes the `.xrs` | `writes_a_sibling_xrs_when_no_output_is_given`; also hand-verified |
| 2 | Matches the golden file byte for byte | `golden.rs::golden_output_matches` over `tests/golden/` |
| 3 | rustc compiles it with **zero stderr** | `hello.rs::compile_and_run` asserts `stderr.is_empty()` |
| 4 | Binary exits zero, prints `hello\n` | `hello_sdx_transpiles_compiles_and_prints_hello` |
| 5 | Comments reach the generated Rust | `comments_survive_into_the_generated_rust` — leading, inline and end-of-file |
| 6 | Every node carries a byte span | `tokens_carry_byte_spans`, `nodes_carry_spans_that_index_the_source` |
| 7 | Two runs byte-identical | `transpiling_twice_produces_identical_bytes`, `emission_is_deterministic` |
| 8 | A tab is rejected, naming the line | `rejects_a_tab_in_indentation_with_a_located_diagnostic` → `tab.sdx:2:1` |
| 9 | An inconsistent dedent is rejected | `rejects_an_inconsistent_dedent_with_a_located_diagnostic` |
| 10 | `docs/COMPILER.md` records the bake-off | §1 — six axes measured, two-condition revisit trigger |
| 11 | No `.rs` over the line cap | `line_cap.rs`; largest is `redox-lexer/src/lib.rs` at 575 |
| 12 | Full gate green | all six moon tasks exit 0 |

### What the bake-off actually found

Doc 02 §5.2's prior was that LALR(1) "fits indentation-sensitive grammars badly". The
measurement is stronger: **LALRPOP cannot produce `Indent`/`Dedent` at all**, so that arm
needs a hand-written pre-lexer regardless, and the generated parser replaces only the
smaller half of the front end — 26 grammar lines plus a 64-line pre-lexer, against a
1,002-line generated module and 46 compiled dependency crates, where the hand-written path
has **zero** third-party dependencies. It also reports **one** error where recursive descent
reports every bad item, measured directly. Full table in `docs/COMPILER.md` §1.

### Three defects found by running it, not by inspection

1. **Structural tokens absorbed trivia.** `Indent`/`Dedent`/`Newline` claimed pending
   comments, so a comment between two top-level items, or at end of file, attached to a
   token the parser never reads — and vanished. Found because `comments.sdx` came out
   missing its last line. Fixed in `redox-lexer`, with two regression tests.
2. **A stray trailing blank line.** A source file's own final newline arrives as a
   `BlankLines` trivium, which appended a blank line to every generated file. Found by the
   end-to-end test comparing against §3's stated output. The emitter now normalises to
   exactly one terminating newline.
3. **`rustc` is not on `PATH` inside `blaze test`** — the test got NixOS's `stub-ld` shim.
   Fixed properly rather than skipped: `build.rs` bakes the absolute path in at build time,
   and both rustc-invoking tests now pass under blaze as well as cargo. Recorded in
   `AGENTS.md` with two further traps: stale blaze state producing bogus link errors in a
   crate that was never touched, and the ANSI-escape issue that made a grep-based clippy
   check report a **false green** while two real errors were live.

### Deviations from the plan as written, and why

- **Indentation width is not fixed by the lexer.** §4 leaf 2 says four spaces, but every
  example in docs 01–05 uses two. The lexer accepts any consistent width and rejects only
  inconsistency, which makes the discrepancy moot for the compiler; `docs/COMPILER.md` §4
  records it and assigns the canonical-width decision to plan 34, the formatter.
- **`Stmt` carries trivia too.** §4 leaf 3's sketch put trivia only on `FnDecl`, but inline
  comments attach to statements, so leaf 2's "never discarded" property would have held only
  partly. `Stmt` gained `leading`/`trailing` to make it true end to end.
- **A second fixture was added.** `examples/comments.sdx` proves the trivia property in all
  three positions; §3's `hello.sdx` alone cannot.
- **`crates/redox-cli/build.rs` was added**, which the plan did not anticipate — see defect 3.
