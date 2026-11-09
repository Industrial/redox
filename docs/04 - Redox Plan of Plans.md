# Redox — Plan of Plans

Status: active index
Tracks which plans, taken together, implement `docs/01 - Redox.md` as corrected and
extended by `docs/02 - Redox Update Blueprint.md` and `docs/03 - Redox Surface Design.md`.

**This document is an index only.** It does not author plan content — each row becomes its
own plan document, `docs/<doc> - <plan> - <Name>.md`, authored when its turn comes. Update the
**Status** column here as each plan is authored, executed and shipped; do not let plan
detail leak back into this file.

Convention adopted from Emerald's `history/2026-09-08T174011Z-plan-of-plans.md`, including
the three habits that make it worth keeping (doc 02 §3.1):

1. **Index only.** Scope is one cell, not a section.
2. **Correct stale labels in place**, with a date and the evidence that corrected them.
3. **Never delete a superseded row.** Mark it `superseded` and name what replaced it. The
   record of what was tried and abandoned is more valuable than a tidy list.

---

## Status legend

| Status | Meaning |
|---|---|
| `not-started` | Row exists, no plan authored yet |
| `planned` | Plan document authored under `docs/<doc> - <plan> - …`, not yet executed |
| `in-progress` | Executing |
| `partial` | Some of the row shipped, the rest named in its own cell |
| `done` | Shipped, gate green, reviewed |
| `superseded` | Replaced; row kept with a pointer to its successor |

`partial` was added when plan 13 landed doc 18, because that one plan delivered pieces of
sixteen later rows and *deleted* pieces of several more. Calling such a row `not-started`
would have been false and calling it `done` worse, and the alternative — splitting each row in
two — would have renumbered the register, which rule 2 forbids. A `partial` row says in its own
cell what shipped, what was deleted, and what is outstanding; nothing else is allowed to be
vague about it.

### What plan 13 did to this table

Doc 18 is a surface, not a feature, so it crossed the register rather than occupying one row.
Nineteen rows moved: **eight to `done`** (17, 24, 25, 26, 27, 31, 35, 36), **three to
`superseded`** (19, 32, 37), and **eight to `partial`** (14, 18, 21, 22, 28, 29, 30, 38). Every
one of them names what changed in its own cell.

The three `superseded` rows are worth reading together, because each was retired by *deletion*
rather than replacement: doc 18 removed the construct that needed the plan. Plan 19's eight
call-grammar rules dissolved into juxtaposition, plan 32 had nothing left to broadcast over
once `.(` and the `.+`-family left, and plan 37's `defer` closed doc 03 §23 item 1 — the one
disclosed exception to doc 01 §36 — by ceasing to exist.

## Plan records

`docs/<doc> - <plan> - <Name>.md`, one per row. A plan takes the next free document number
and carries its plan number too, so the docs tree reads in authoring order while each plan
is still identifiable by its row here:

| Plan | Document |
|---|---|
| 01 | `docs/05 - 01 - Vertical Slice.md` |
| 02 | `docs/07 - 02 - Lexical Surface.md` |
| 04 | `docs/09 - 04 - Emitter Determinism.md` |
| 03 | `docs/08 - 03 - Parser Core.md` |

**`docs/05 - 01 - Vertical Slice.md` is the authored exemplar** — every subsequent plan
follows its shape:

| § | Section | Purpose |
|---|---|---|
| 1 | What this plan is | Why this scope, and why not larger or smaller |
| 2 | Verified preconditions | Anything measured rather than assumed, with the command and its output |
| 3 | Concrete proof | The executable acceptance test, with the exact expected output |
| 4 | Leaves | One subsection per unit of work, implementable without further design |
| 5 | Acceptance criteria | Numbered, each independently checkable |
| 6 | Out of scope | Every deferred item named, with the successor plan that owns it |
| 7 | Gate | The commands that must pass |
| 8 | Emerald reuse | What was lifted, what was consulted, what was deliberately not taken and why |
| 9 | Decision log | Choices *and their reasons*, including the ones taken against a prior |

Two conventions worth stating because they are what make the format useful rather than
ceremonial. **§2 exists so that contingencies get removed rather than carried** — plan 01's
§2 settled doc 01 §1.1's open question about rustc's input-extension handling with a single
probe. And **§6 may not say "later"**: a deferred item without a named successor plan is a
gap, not a decision, which is Emerald's "nothing is left open without a named place it gets
resolved" rule (doc 02 §3.3).

The `From Emerald` column below is load-bearing, not decorative. Doc 02 §2 found that of
Emerald's ~107k lines the Redox-relevant surface is roughly 6.5k — `emerald-driver`,
`emerald-lsp`, `emerald-fmt`'s goal and the fuzz harness — plus `emerald-mcp`'s 392 lines,
which doc 02 §9.3 held outside that figure as optional and plan 50 now takes, plus process
and grammar methodology. A row claiming reuse must name the file; a row claiming none says
`—`, and that is the expected answer for most of the front end, because Redox's lexer is
indentation-sensitive and its parser is hand-written where Emerald's are neither.

---

## Milestone 1 — Skeleton

A program goes from `.sdx` to a running binary. Nothing about the language is interesting
yet; everything about the pipeline is in place, including the two things doc 02 identifies
as expensive to retrofit: spans and trivia.

| # | Plan | Scope | From Emerald | Depends on | Status |
|---|---|---|---|---|---|
| 01 | `vertical-slice-hello` | Workspace scaffold; off-side lexer with trivia, spans and adjacency; hand-written parser for one item form; Rust emitter; `redox build`; compile-and-run test. Parser bake-off against LALRPOP as a leaf (doc 02 §3.5, doc 03 §23 item 6) | `crates/emerald-cli/tests/hello_em.rs` (harness shape); workspace `Cargo.toml` `[workspace.package]` inheritance | — | `done` 2026-11-08 — 8 crates, 50 tests, full gate green. Bake-off ran for real and chose hand-written: LALRPOP **cannot** emit `Indent`/`Dedent` at all, so it needs a hand-written pre-lexer anyway, and it reports one error where recursive descent reports every bad item (`docs/COMPILER.md` §1). Also settled doc 01 §1.1 (rustc accepts `.xrs` directly, §3) and recorded doc 01 §3.1's four-vs-two-space discrepancy (§4) |
| 02 | `lexer-complete` | Full off-side rule: continuation suppression inside delimiters and after trailing operators, INDENT/DEDENT, tab rejection, `#` longest-match table (doc 03 §4.1), interpolation lexing, `"""` with line-number preservation | `crates/emerald-parser/src/heredoc.rs` (consulted for two *requirements* — interpolation inside a multi-line body, and line-number preservation — not for code) | 01 | `done` 2026-11-08 — `docs/07 - 02 - Lexical Surface.md`. Lexer core rewritten from a line loop to a cursor, which both features demanded. 117 tests. Four defects found by running it, incl. that doc 03 §5.3 puts `|>` at the *start* of a continuation line, so continuation needed lookahead as well as trailing detection |
| 03 | `parser-core` | Items, statements, expression grammar with a Pratt loop on Rust's precedence table, multi-error recovery at item and statement boundaries | `crates/emerald-parser` plan 26's `errors: &mut Vec<_>` threading, as a *contrast* — a hand-written parser recovers natively | 01, 02 | `done` 2026-11-08 — `docs/08 - 03 - Parser Core.md`. Doc 01 §31 Phase 1 now prints `30` through real rustc. 132 tests. Three defects found by running it: a declared return type needs a tail expression not a statement; plan 01 s parenless call was too narrow for its own successor; and a third blaze shim, fixed by *verifying* a rustc candidate rather than trusting either `PATH` or `RUSTC` |
| 04 | `emitter-determinism` | Deterministic, readable emission; golden-test harness with `--bless`; no rustfmt in the contract (doc 02 §5.3) | — | 03 | `done` 2026-11-08 — `docs/09 - 04 - Emitter Determinism.md`. Readability became an assertion: rustfmt is used as a *measurement* in a test, and every example is already rustfmt-clean. The check had to run outside the repo, since this repo own rustfmt.toml governs Redox source, not Redox output. **Milestone 1 complete** |

## Milestone 2 — Diagnostics and source maps

Doc 02 §6.5 calls this the largest single gap in doc 01: "pass Rust errors through" has no
mechanism, so a user debugs generated `.xrs` they never wrote. Placed before language growth
deliberately — Emerald's plan 22 found `TypeExpr` carried no span at all, twenty-one plans
after its diagnostics plan, and settled for a scoped workaround because the full fix was
judged too large by then.

| # | Plan | Scope | From Emerald | Depends on | Status |
|---|---|---|---|---|---|
| 05 | `diagnostics-miette` | `redox-diag`; `miette` graphical rendering; stable `RDX` code catalogue (doc 03 §17); one report per recovered error, single non-zero exit | `crates/emerald-cli/src/main.rs:47-81` (`SemaError`'s `miette::Diagnostic` impl) and `report_driver_error`'s per-error loop | 03 | `done` |
| 06 | `source-map` | `SourceMap` beside the `.xrs` as a `.xrs.map` sidecar, one record per generated line. **Line-preserving emission was attempted and rejected** — braces need their own lines and plan 04's rustfmt guarantee outweighs the simplification (plan 06 §2.1 corrects doc 02 §6.5 item 1) | — (Emerald's analogue is DWARF via `inkwell`, one layer down) | 04, 05 | `done` |
| 07 | `rustc-remapping` | `redox check`: **rustc** run with `--error-format=short` (not `cargo check --message-format=json` — there is no Cargo project yet, and the five fields needed fit on one line; plan 07 §2.2), spans rewritten through the map, re-rendered against the `.sdx`. Proved by executed tests with a deliberate error on a known line — exact to the **column** on a verbatim line, to the **statement** on a restructured one (§2.1) | `crates/emerald-driver/build.rs` (Cargo NDJSON artifact-stream parsing, same technique, different purpose) | 05, 06 | `done` |
| 08 | `run-expectations` | The third corpus: what a program **prints**, in a `.stdout` beside it, blessable. Plan 05 delivered the UI harness early, so this plan is the leftover — and the leftover mattered: it moved three expectations out of Rust string literals, added a machine check against reintroducing them, and caught two emitter defects in string handling that had shipped since plan 02. **`redox test` was rejected**, not deferred (doc 01 §27 fixes the CLI surface; plan 08 §6.1) | — | 05 | `done` |

## Milestone 3 — Real projects

| # | Plan | Scope | From Emerald | Depends on | Status |
|---|---|---|---|---|---|
| 09 | `cargo-integration` | Doc 02 §6.6 option (a): `build.rs` + `OUT_DIR` + `include!`, with `redox_driver::compile_dir` as the library a build script calls. Proved by `examples/cargo-project/`, a real crate that `cargo run` builds. **Doc 02's span claim corrected** — `cargo build` reports against `OUT_DIR` because nothing reads the map; `redox check` is the remapping workflow (plan 09 §2.2). Found that Redox cannot yet fill a `mod`, which needs plan 24's `export` | — (Emerald has no generated intermediate source) | 06, 07 | `done` |
| 10 | `incremental-cache` | `blake3` content-addressed cache behind `compile_dir`, with the compiler's own binary hashed into every key and a `CacheReporter` that has no opinion on stdout. The governing rule, quoted from Emerald: a MISS falls back verbatim, so a hit is strictly an optimization — asserted by three tests that would pass with the cache deleted | `crates/emerald-driver/src/cache.rs` — doc 02 §7.1's highest-value reuse, taken as a design and deliberately smaller (one query, not three) | 09 | `done` |
| 11 | `cli-surface` | One command table that dispatch, the usage text and the test all read, so they cannot disagree — the usage was a hand-written const and dispatch a hand-written match, two lists nothing kept in step (plan 11 §2.1). A **planned** command names its plan and is distinct from an unknown one. `clap` rejected, with the condition that would change it | `crates/emerald-cli/src/main.rs` dispatch shape, minus eleven subcommands Redox declines | 07 | `done` |
| 12 | `watcher` | `redox watch` on `notify`, surviving compile errors, with **no timing heuristic** — plan 10's cache makes redundant work cheap. Two event filters, by path and by kind, and **both are necessary**: a path-only filter made the build's own read of its input retrigger the build, 97,442 times in six seconds (plan 12 §10.1, which corrects §1.1) | — (Emerald has no watch mode) | 10, 11 | `done` |

## Milestone 4 — Specification-correct (doc 03 Tier A)

Corrections to a published specification, two of which are cases where current Redox source
would mean something other than the Rust it resembles. These land before any surface growth
regardless of schedule (doc 03 §20).

| # | Plan | Scope | From Emerald | Depends on | Status |
|---|---|---|---|---|---|
| 13 | `haskell-surface` | **The surface is doc 18's.** Comments become `--`/`{- -}`/`-- |`/`-- !`; a declaration becomes a `::` signature line plus an equation; application becomes juxtaposition. The generated Rust is **byte-identical** — the only golden diff is a comment — which is doc 18 §1's rule measured rather than asserted. Deleted `stmt_call`, `parse_signature`, `parameter` and `POSTFIX_POWER`; `ranges-rust-spelling` is subsumed, since doc 18 §8 keeps Rust's `..`/`..=` for the same reason doc 03 decision 34 did | — | 12 | `done` |
| 14 | `paths-colon-colon` | `::` everywhere including imports; `.` means field or method only. Corrects doc 01 §22 **Partial.** Doc 18 §5.2's tight `::` is a path and `.` is field or method only, both implemented and tested; `import` is not parsed, so the import half is outstanding. **Scope widened and plan authored, 2026-11-09**, as `docs/19 - 14 - Paths, Imports and Modules.md`: doc 03 §13.5's module organisation — inline blocks, glob imports and brace-nested trees — had **no row at all**, and `import` is where it belongs. Doc 18 §7 retires `mod` for `module`, so the two arrive together. **Done 2026-11-09.** `import` in all five of §13.1's forms, `module` bodiless and with a body, nested, with per-module test gathering. Closed plan 09's recorded gap: `examples/cargo-project/` now uses the module shape its own note said it could not | — | 03 | `done` |
| 15 | `await-postfix` | Postfix `.await`. Corrects doc 01 §24. **Plan authored 2026-11-09** as `docs/20 - 15 - Postfix Await.md`. One leaf: the postfix machinery already exists for `?`, a field, a tuple index and a method call, and doc 03 §14.3 wants no lowering. Row 39 keeps `async fn` and `async` blocks, so every use of `.await` is a rustc error until it lands — disclosed in the plan's §6. **Done 2026-11-09.** Found that `?.` is one token — doc 03 §12.3's optional chaining — so `x?.await` is a parse error rather than two postfixes, and the collision is now pinned by a test so row 28 cannot change it quietly | — | 03 | `done` |
| 16 | `module-scope-bindings` | Top-level bindings require a type and lower to `const`/`static`; `let` is illegal at Rust module scope. Corrects doc 01 §4.3. **Scope widened and plan authored 2026-11-09** as `docs/21 - 16 - Module-Scope Bindings.md`: doc 18 §2.2's `const fn` is the same keyword in the same position as §13.4's `const`, and splitting them would mean reading the token in one plan and acting on it in another. Row 39 keeps `async`. **Done 2026-11-09**, with `RDX0106` naming §13.4's correction when an author writes doc 01 §4.3's `PI = 3.14159`. Found that the example corpus cannot demonstrate a *lint*: it requires rustc silence, and a lowercase `const` is a rustc warning | — | 03 | `done` |
| 17 | `comments-as-trivia` | `#`, `###`, `##`, `##!` as trivia attached to AST nodes and re-emitted. Corrects doc 01 §4.1 **Done**, with doc 18 §5.1's spellings in place of `#`: trivia is attached to AST nodes and re-emitted, which plan 02 delivered and every plan since has kept — doc 02 §10.1's rule, and the defect this row names Emerald for | `crates/emerald-fmt/src/lib.rs`'s disclosed comment-loss defect, as the anti-pattern to avoid | 02, 04 | `done` |
| 18 | `attributes-passthrough` | `#[...]`/`#![...]` verbatim; `derive` clause. Closes doc 02 §10.2's critical gap **Partial.** `deriving` is a clause of the declaration (doc 18 §2.4) and lowers to `#[derive(…)]`; verbatim `#[…]` and `#![…]` passthrough is outstanding. **Plan authored 2026-11-09** as `docs/22 - 18 - Attributes.md`. One leaf: plan 02 already lexes `TokenKind::Attr`, so the work is deciding what an attribute attaches to — and the answer is **trivia**, which keeps source order with comments and reaches nine item kinds and every statement at once. **Done 2026-11-09** in two `TriviaKind` variants, two `emit_trivia` arms and a deletion — `TokenKind::Attr` became unreachable and is gone. Attributes on a *field* or *variant* remain outstanding: `FieldDecl` and `Constructor` have no `leading` | — | 03 | `done` |

## Milestone 5 — Core language (doc 03 Tier B)

| # | Plan | Scope | From Emerald | Depends on | Status |
|---|---|---|---|---|---|
| 19 | `calls-parenless-canonical` | Doc 03 §5.1's eight rules: adjacency lexing, comma below every operator, terminator set, six required-parens cases, `RDX0055`. The most intricate region of the parser **Superseded by plan 13.** Doc 18 §2.1 removed the ambiguity instead of resolving it: juxtaposition made statement position unremarkable, and the eight rules, the terminator set, the six required-parens cases and `RDX0055` all dissolved. §7.1 is the accounting | Emerald plan 90's full analysis of the same problem — the two candidate strategies and why statement-position-only loses | 02, 03 | `superseded` |
| 20 | `control-flow` | `if`/`unless`/`else`, `while`/`until`, `for`, postfix forms, `cond`, `loop`, `'label` on blocks and loops, `break` with value. **Plan authored 2026-11-09** as `docs/23 - 20 - Control Flow.md`, and the row is **smaller than it was written**: doc 18 deleted `cond` (guards do its work), `unless` (`if not`) and the postfix forms (their expanded forms). What is left is `if`/`else`, `while`/`until`, `for x <- xs`, `loop`, `do` with labels, and `break` with a value | — | 03 | `planned` |
| 21 | `switch-and-patterns` | `switch` arms and `when` guards; relational, `and`/`or`/`not`; named and anonymous record patterns; slice, as- and range patterns; no synthesized wildcard **Partial.** `case … of` with `|` guards, `or` alternation and `..`/`..=` ranges are implemented (doc 18 §2.3); relational, `and` and `not` patterns are *deleted* by §2.3 in favour of guards. Record, slice and as-patterns are outstanding, and are plan 33's | — | 20 | `partial` |
| 22 | `is-and-guard` | `is` as binding-aware pattern test (`matches!` / `if-let` / let-chain); `guard` onto let-else; derived `while let` **Partial.** `is` is deleted by doc 18 §3.1 in favour of `<-`, which is implemented in statement position. `<-` in condition position and `guard` onto let-else are outstanding | — | 21 | `partial` |
| 23 | `strings-and-literals` | Interpolation with inline format specs; raw, char, byte and C-string literals; `"""` dedent; `##`/`##!` doc comments | — | 02, 17 | `not-started` |
| 24 | `data-declarations` | Structs with block init and trailing spread; field punning; primary constructors; enum struct variants and discriminants; granular visibility; type aliases; unit and tuple structs **Done** as doc 18 §2.4 and doc 03 §9.6: `data`, `newtype`, `type`, record literals with punning and update, and the four visibility markers including per-field. Block initialisers and primary constructors are *deleted* by §2.4, which takes `RDX0052` and `RDX0053` with them | — | 18 | `done` |
| 25 | `traits-and-impls` | Default method bodies, associated types and constants, supertraits, generic impls with bounds, `impl Trait` in argument and return position **Done** as doc 18 §2.5: `class` is `trait` and `instance` is `impl`, with default method bodies, associated types and constants, supertraits and generic impls with bounds. `impl Trait` in either position needs nothing — a type is verbatim text (doc 03 §14.1) | — | 24 | `done` |
| 26 | `generics-and-where` | Generic parameters and bounds; `where` as a bound clause in signature position, disambiguated from `where` locals by position **Done** as doc 18 §§2.2 and 5.3: the generic list is synthesised from the types and the bounds are read off `=>`, so neither is written. `where` keeps one role — the position disambiguation this row describes is deleted rather than implemented | — | 25 | `done` |

## Milestone 6 — Full surface (doc 03 Tier C) — **v1.0**

Without these Redox is a syntax skin; with them it is the language docs 01–03 describe.
v1.0 is reached when every row from 01 to 33 is `done`.

| # | Plan | Scope | From Emerald | Depends on | Status |
|---|---|---|---|---|---|
| 27 | `pipeline-and-sections` | `\|>` as a receiver pipeline; operator sections with the auto-ref lowering for comparisons; `(-1)` versus `(- 1)` by adjacency **Done** as doc 18 §4: the pipeline, and all six sections including the method form for comparisons. `(-1)`-versus-`(- 1)` is *deleted* — §4 makes a leading `-` negation always, and the same rule covers `&`, `*` and `!` | — | 19 | `done` |
| 28 | `option-result-operators` | `??`, `??=`, `catch`, `?.` via `?`-in-a-closure, `try` blocks. No implicit clone **Partial.** `??` lowers to a `match` (doc 03 §12.1) and `?` passes through; `??=` is *deleted* by doc 18 §7 in favour of its expanded form. `catch`, `?.` and `try` blocks are outstanding | — | 22 | `partial` |
| 29 | `comprehensions-and-collections` | Bracket comprehensions and collection-`if`/`for`; literal-shape collect defaults with `into` override; spread; `%w`; set literal; array repeat **Partial.** Doc 18 §3.2's comprehension and the list literal are implemented, and the ascribed-literal lowering `C::from([…])` covers the set and map literals this row names — both the collection-`if`/`for` level and the `into` override are *deleted* by §3.2. `%w` and array repeat are outstanding | — | 19, 21 | `partial` |
| 30 | `with-and-use` | `with` dot-shorthand mutation block, subject evaluated once; `use` callback flattening with `RDX0044` on a `return` in the tail **Partial.** `use` is `<-` and its callback flattening is implemented (doc 18 §3.1); `with` is *deleted* by §7. `RDX0044` on a `return` in a callback tail is outstanding | — | 19 | `partial` |
| 31 | `functions-advanced` | Multi-clause definitions merged to one `fn` plus `match`; labelled arguments erased in declaration order; `where` locals; `...` forwarding; `&:` method references **Done** as doc 18 §2.3 and §2.2: multi-clause definitions merge to one `fn` plus a `match`, and `where` locals lower to `let` bindings before the body. Labelled arguments, `...` forwarding and `&:` method references are all *deleted* by §7 — the last replaced by `(|> name)`, a section derived from the rule | — | 26 | `done` |
| 32 | `broadcasting` | `f.(xs)` and `.+`-family elementwise operators, borrow marker mandatory, result is an iterator **Superseded.** Doc 18 §7 retires `.(` and the `.+`-family outright, so there is nothing left to broadcast over | — | 29 | `superseded` |
| 33 | `destructuring-and-splat` | Tuple, slice and struct destructuring; rest patterns; splat in argument position | — | 21, 29 | `not-started` |

## Milestone 7 — Tooling (doc 03 Tier D)

Doc 02 §9 fixes this milestone's shape. Redox's *CLI* ships far less than Emerald's — plan
11 keeps six commands and Cargo owns the rest — but four pieces of Emerald's tooling surface
are worth more to Redox than doc 01 credits, and all four are **committed rows here, not
optional extras**:

| Thing | Plan | Source |
|---|---|---|
| Language server (LSP) | 40 `lsp` | doc 02 §9.1 |
| tree-sitter grammar | 41 `tree-sitter-grammar` | doc 02 §9.4 |
| Editor syntax and clients — Vim, Neovim, VSCode, Cursor, Helix, Zed | 49 `editor-integrations` | doc 02 §9, and §5.4's `editors/` |
| MCP server | 50 `mcp-server` | doc 02 §9.3 |

**Doc 02 §9.3's "optional" label is corrected here, 2026-11-08.** MCP is a committed row: an
agent-facing `redox check` returning plan 05's diagnostics as structured JSON is how this
repo's own harness should read them, and `.mcp.json` already exists for the server to
register in. The same reasoning promotes editors — for an indentation-sensitive language
with brand-new syntax, live diagnostics and highlighting are close to a precondition for
anyone trying it (doc 02 §9.1) — so plan 49 names its clients rather than leaving "editors"
as a gesture.

| # | Plan | Scope | From Emerald | Depends on | Status |
|---|---|---|---|---|---|
| 34 | `formatter` | `redox-fmt`: trivia-preserving, canonicalizing per doc 03 §16, and therefore the migration tool. Doubles as the answer to Emerald's 113-missed-fixture cutover | `crates/emerald-fmt`'s goal and its approach's disclosed failure — a pure AST pretty-printer loses every comment | 17, 19 | `not-started` |
| 35 | `test-blocks` | `test "…"` to `#[cfg(test)]` module with `#[test]` fns; deterministic name slugs; `assert` family **Done** as doc 03 §15: `test "…"` becomes one `#[cfg(test)]` module per file with deterministic name slugs and an index on collision, and `assert` reads the operator the author wrote. The run corpus compiles the generated module as a test binary and runs it | `crates/emerald-cli/tests/examples.rs` compile-and-run-and-assert-stdout harness | 23 | `done` |
| 36 | `typed-domain-errors` | `errors` declaration generating the enum plus `Display`, `Error` and `From` impls. `std` only **Done** as doc 03 §12.5: the enum, `Display`, `Error` and a `From` per `from` row, naming nothing outside `std` | — | 28 | `done` |
| 37 | `defer` | Scope-exit blocks, LIFO, via a generated `core`-only `Drop` guard. **Blocked on doc 03 §23 item 1** — the one disclosed exception to doc 01 §36 **Superseded.** Doc 18 §3.1 deletes `defer`, which closes doc 03 §23 item 1 — the one disclosed exception to doc 01 §36 — by removing the construct that needed it | — | 20 | `superseded` |
| 38 | `panic-and-lints` | `todo`/`unimplemented`/`unreachable`/`panic`/`assert` spellings; `RDX1001` `unwrap` discouragement **Partial.** `todo`, `unimplemented`, `unreachable`, `panic` and the `assert` family are implemented (doc 03 §12.6); `RDX1001`'s `unwrap` discouragement is outstanding | — | 23 | `partial` |
| 39 | `async` | `async` functions and blocks, postfix `.await`, no executor | — | 15, 26 | `not-started` |
| 40 | `lsp` | `redox-lsp` on `lsp-server` + `lsp-types` (doc 02 §5.5): diagnostics on change, semantic tokens, document symbols, and a formatter hook onto plan 34. Rustc errors arrive already remapped to the `.sdx` through plan 07, so the server reports them against the file the user wrote. Editor-agnostic by construction — plan 49 ships the clients. Emerald does this in 545 lines; Redox's is simpler, having no symbol table to build | `crates/emerald-lsp` (545 lines, directly adaptable) and plan 90's warning to purge `(`-assuming heuristics first | 07, 34 | `not-started` |
| 41 | `tree-sitter-grammar` | `tree-sitter-redox/` per doc 02 §5.4: `grammar.js` plus an external scanner for INDENT/DEDENT, the generated parser, `highlights`/`injections`/`locals` queries, and a corpus suite — which doubles as an independent check on the grammar's shape (doc 02 §9.4). The cost is disclosed, not hidden: an indentation-sensitive tree-sitter grammar is genuinely fiddly, and it is a second grammar to keep in sync with plan 46's inventory | Emerald plan 24's `tree-sitter-emerald` layout and corpus-test convention | 19 | `not-started` — **scope narrowed 2026-11-08**: the editor configs this row carried moved to plan 49, which names Vim, Neovim, VSCode and Cursor explicitly and is too large to ride along with a grammar |
| 42 | `fuzzing` | `lex_source`, `parse_program`, `emit_program` and a generated-program target whose oracle is *the generated Rust must parse* — a stronger oracle than Emerald had | `crates/emerald-fuzz-smoke` architecture: logic in a stable crate, `fuzz_targets/*.rs` as three-line wrappers, zero dependencies | 04 | `not-started` |
| 43 | `test-suite-expansion` | Property tests: lexer indentation round-trip, formatter idempotence, transpile stability, `parse(fmt(x)) ≡ parse(x)` modulo trivia | — | 34, 42 | `not-started` |
| 44 | `zero-cost-benchmarks` | Generated versus hand-written Rust, compiled identically, asserted within noise; transpile throughput and watch latency. Doc 03's zero-cost claim is a claim until measured | `benchmarks/REPORT.md`'s methodology discipline — N programs, N runs, methodology disclosed, losses reported | 09 | `not-started` |
| 49 | `editor-integrations` | `editors/` per doc 02 §5.4 — one directory per client, each independently installable, each CI-checked. **Neovim**: `.sdx`/`.xrs` filetype detection, plan 41's grammar registered with `nvim-treesitter`, plan 40's server wired through `vim.lsp`. **Vim**: `syntax/sdx.vim` and `ftplugin/sdx.vim`, regex highlighting only — the fallback where neither tree-sitter nor a built-in LSP client exists. **VSCode and Cursor**: one extension, shared contribution points — a TextMate grammar so a cold open is coloured, an LSP client over `vscode-languageclient`, and `redox fmt` as the document formatter; Cursor consumes the VSCode extension unchanged, and that is **verified by installing it, not assumed**. **Helix** and **Zed**: `languages.toml` and an extension manifest, both tree-sitter-native. Install steps live beside each client, not buried in a README | Emerald's VSCode/Neovim/Helix/Zed configs (doc 02 §9) — layout reused; Vim and Cursor are Redox additions | 40, 41 | `not-started` |
| 50 | `mcp-server` | `redox-mcp`: the driver exposed over the Model Context Protocol on `rmcp` + `schemars` + `tokio` (doc 02 §5.5, §9.3). Tools: `redox_check`, `redox_build`, `redox_fmt`, and a source-map lookup answering "which `.sdx` line is this `.xrs` line" from plan 06's map. Each returns plan 05's diagnostics as **structured JSON, never scraped stderr**. Stdio transport only; no network listener. A `.mcp.json` entry is committed so this repo's own agents get it | `crates/emerald-mcp` (392 lines — the driver-over-MCP shape) | 05, 06, 11 | `not-started` |

## Milestone 8 — Closing

| # | Plan | Scope | From Emerald | Depends on | Status |
|---|---|---|---|---|---|
| 45 | `reachability-residue` | Spell `unsafe`, `extern`/FFI, `union`, raw identifiers, macro definition. Doc 01 §26 names FFI as a requirement, so the escape hatch is not an acceptable resting place (doc 03 §18, §23 items 2–4) | — | 25 | `not-started` |
| 46 | `grammar-inventories` | The two graded inventories doc 02 §4.1 calls for: MoonScript as `KEEP`/`MODIFY`/`REMOVE`, Rust as `PASSTHROUGH`/`SUGARED`/`UNREACHABLE`. Doc 03 §18 is a summary, not a substitute | `spec/GRAMMAR.md` — the methodology itself, doc 02 §4.1's highest-value design transfer | 45 | `not-started` |
| 47 | `status-and-declines` | `docs/STATUS.md` with measured, reproducible numbers and a visible gap table; `docs/DECLINED.md` from doc 03 §22 | `README.md`'s "none of this is rounded up" posture and `examples/README.md`'s gap-and-bug tracking | 44 | `not-started` |
| 48 | `conformance-self-hosting` | The compiler's own source expressed in Redox, **without the inline-Rust escape hatch** — a measurement of expressiveness, not an architecture goal (doc 02 §4.8). Generated `.xrs` committed so the bootstrap never breaks | — (Emerald never self-hosted at 107k lines; weak evidence, but the reframing argument is Redox-specific) | 45, 46 | `not-started` |

---

## Ordering commitments

Four constraints on reordering, each with a reason that cost somebody something:

1. **Spans and trivia land in plan 01, not later.** Emerald's plan 22 found `TypeExpr`
   carried no span twenty-one plans after the diagnostics plan, and the full fix was by then
   "judged too large/risky" — so it shipped a scoped workaround and a disclosed residual gap.
2. **Milestone 2 precedes Milestone 5.** Retrofitting a source map onto an emitter not built
   for one is the same class of rework, and doc 03's multi-line expansions (`use`, `defer`,
   `with`, multi-clause merging) widen the source-to-generated gap considerably.
3. **Milestone 4 can interleave anywhere after plan 03.** Its rows are corrections to a
   published specification, not features, and two of them change what existing source means.
4. **Plan 34 should not drift late.** The formatter is the migration tool (doc 02 §4.5), and
   Emerald's plan 71 cutover missed 113 fixtures embedded as string literals in Rust test
   modules. Redox's corollary is already a convention: **every `.sdx` fixture is a file, and
   tests reference files.**

## Code architecture

Every plan's source obeys `docs/06 - Redox Code Architecture.md`, which is normative and
machine-checked. Its central rule — **one construct per file**, named after the construct —
is enforced by `crates/redox-cli/tests/one_construct_per_file.rs` and therefore part of the
gate below rather than a review convention. A plan that adds a construct also adds a file;
no plan may introduce a `types.rs`, `helpers.rs` or any other bucket.

## Gate

Every plan clears, before it is `done`:

```bash
moon run :check          # blaze check — fast inner-loop verdict
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # unit, integration, doctests
moon run :cargo-check    # rustc's verdict — AGENTS.md: rustc is the final word
moon run :cargo-test     # cargo nextest run
moon run :format         # treefmt
```

From Milestone 1 onward, additionally: every example `.sdx` transpiles, **its generated Rust
compiles under real rustc**, runs, and its stdout is asserted. Golden text proves stability;
compilation proves correctness; neither substitutes for the other (doc 02 §8.1–8.2).

## Completion

- **v1.0** — rows 01–33 `done`.
- **Feature-complete** — rows 01–44 plus 49 and 50 `done`. Those two are Milestone 7 rows
  added after the first numbering pass (2026-11-08), and they carry the next free plan
  numbers, so they sit at the end of their milestone's table rather than beside plan 41.
  They are not optional.
- **Specification-closed** — rows 01–50 `done`; `docs/03 - Redox Surface Design.md` §23 has
  no open items, and doc 03 §18's residue list is empty.
