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

### What doc 30 will do to this table

`docs/30 - Bind.md`, decided and **converted 2026-11-09** in four commits, makes `<-` mean
`await` and nothing else — doc 18 §3.1 had given it four jobs and kept Haskell's meaning in
only one of them. This note is here so a reader of a row below is not surprised by a spelling
the row does not mention.

| Row | What doc 30 changed | Status |
|---|---|---|
| 13 `haskell-surface` | The callback bind is `with x = f` | `done` |
| 15 `await-postfix` | Postfix `.await` is **deleted** — every await binds | **`superseded`**, the fourth |
| 20 `control-flow` | `for x <- xs` is `for x in xs` | `done` |
| 22 `is-and-guard` | A condition and a `guard` take `=` | `done` — three `expect` calls and one scan |
| 29 `comprehensions-and-collections` | The generator takes `in` | `done` |
| 30 `with-and-use` | `RDX0044`'s message names `with`; the lowering is untouched | `done` |
| 39 `async` | Inherits `<-` as the await form; supplied the `async` marker and block that let one be written | **`done`** — doc 30 §8.4's debt paid 2026-11-09 |

The order was `in`, then `with`, then `=`, then `<-` last, because only then was the token
free. Not one golden moved in the first two steps and one UI expectation's quoted source line
moved in the third — doc 30 §8.1.

**Row 39 carried a debt and has paid it.** Doc 30 §8.4 recorded that `<-` was proved by
eight emit tests and not by a program that runs, because every await needs an `async`
function. Row 39 shipped the marker and `examples/async.sdx` on **2026-11-09**, and that
example brings its own nine-line executor — doc 03 §14.3 provides none, which is the claim
the example exists to demonstrate.

---

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
| 15 | `await-postfix` | Postfix `.await`. Corrects doc 01 §24. **Plan authored 2026-11-09** as `docs/20 - 15 - Postfix Await.md`. One leaf: the postfix machinery already exists for `?`, a field, a tuple index and a method call, and doc 03 §14.3 wants no lowering. Row 39 keeps `async fn` and `async` blocks, so every use of `.await` is a rustc error until it lands — disclosed in the plan's §6. **Done 2026-11-09.** Found that `?.` is one token — doc 03 §12.3's optional chaining — so `x?.await` is a parse error rather than two postfixes. **Superseded 2026-11-09 by `docs/30 - Bind.md`**, which deletes postfix `.await` entirely: every await binds, spelled `x <- f`. The fourth row retired by a later surface decision rather than by being wrong — and the only one retired by *replacement* rather than deletion, since await still exists. The `?.` collision test survived the deletion and moved to row 28's suite, because doc 30 §2.4 makes it matter more rather than less | — | 03 | `superseded` |
| 16 | `module-scope-bindings` | Top-level bindings require a type and lower to `const`/`static`; `let` is illegal at Rust module scope. Corrects doc 01 §4.3. **Scope widened and plan authored 2026-11-09** as `docs/21 - 16 - Module-Scope Bindings.md`: doc 18 §2.2's `const fn` is the same keyword in the same position as §13.4's `const`, and splitting them would mean reading the token in one plan and acting on it in another. Row 39 keeps `async`. **Done 2026-11-09**, with `RDX0106` naming §13.4's correction when an author writes doc 01 §4.3's `PI = 3.14159`. Found that the example corpus cannot demonstrate a *lint*: it requires rustc silence, and a lowercase `const` is a rustc warning | — | 03 | `done` |
| 17 | `comments-as-trivia` | `#`, `###`, `##`, `##!` as trivia attached to AST nodes and re-emitted. Corrects doc 01 §4.1 **Done**, with doc 18 §5.1's spellings in place of `#`: trivia is attached to AST nodes and re-emitted, which plan 02 delivered and every plan since has kept — doc 02 §10.1's rule, and the defect this row names Emerald for | `crates/emerald-fmt/src/lib.rs`'s disclosed comment-loss defect, as the anti-pattern to avoid | 02, 04 | `done` |
| 18 | `attributes-passthrough` | `#[...]`/`#![...]` verbatim; `derive` clause. Closes doc 02 §10.2's critical gap **Partial.** `deriving` is a clause of the declaration (doc 18 §2.4) and lowers to `#[derive(…)]`; verbatim `#[…]` and `#![…]` passthrough is outstanding. **Plan authored 2026-11-09** as `docs/22 - 18 - Attributes.md`. One leaf: plan 02 already lexes `TokenKind::Attr`, so the work is deciding what an attribute attaches to — and the answer is **trivia**, which keeps source order with comments and reaches nine item kinds and every statement at once. **Done 2026-11-09** in two `TriviaKind` variants, two `emit_trivia` arms and a deletion — `TokenKind::Attr` became unreachable and is gone. Attributes on a *field* or *variant* remain outstanding: `FieldDecl` and `Constructor` have no `leading` | — | 03 | `done` |

## Milestone 5 — Core language (doc 03 Tier B)

| # | Plan | Scope | From Emerald | Depends on | Status |
|---|---|---|---|---|---|
| 19 | `calls-parenless-canonical` | Doc 03 §5.1's eight rules: adjacency lexing, comma below every operator, terminator set, six required-parens cases, `RDX0055`. The most intricate region of the parser **Superseded by plan 13.** Doc 18 §2.1 removed the ambiguity instead of resolving it: juxtaposition made statement position unremarkable, and the eight rules, the terminator set, the six required-parens cases and `RDX0055` all dissolved. §7.1 is the accounting | Emerald plan 90's full analysis of the same problem — the two candidate strategies and why statement-position-only loses | 02, 03 | `superseded` |
| 20 | `control-flow` | `if`/`unless`/`else`, `while`/`until`, `for`, postfix forms, `cond`, `loop`, `'label` on blocks and loops, `break` with value. **Done 2026-11-09**, planned as `docs/23 - 20 - Control Flow.md`. The row was **smaller than it was written**: doc 18 deleted `cond` (guards do its work), `unless` (`if not`) and the postfix forms (their expanded forms). What shipped is the two `if` forms told apart by `then`, `while`/`until`, `for x <- xs`, `loop`, `do` with labels, `break` with a value and `continue` — plus **doc 01 §4.3's compound assignment**, which no row owned and without which a counting loop does not terminate (plan §10.1). Plain reassignment stays **unspecified**, disclosed in plan §10.2 rather than designed around | — | 03 | `done` |
| 21 | `switch-and-patterns` | `switch` arms and `when` guards; relational, `and`/`or`/`not`; named and anonymous record patterns; slice, as- and range patterns; no synthesized wildcard **Partial.** `case … of` with `|` guards, `or` alternation and `..`/`..=` ranges are implemented (doc 18 §2.3); relational, `and` and `not` patterns are *deleted* by §2.3 in favour of guards. Record, slice and as-patterns were plan 33's and **shipped 2026-11-09** with it. The **tuple-variant** form — `Some v` — was named by no row at all and shipped with plan 22 (its §10.1). Relational, `and` and `not` patterns are *deleted* by doc 18 §2.3, with `RDX0042` and `RDX0043`; doc 03 §6.3's anonymous record patterns go with them. **Nothing is outstanding** | — | 20 | `done` |
| 22 | `is-and-guard` | `is` as binding-aware pattern test (`matches!` / `if-let` / let-chain); `guard` onto let-else; derived `while let` **Partial.** `is` is deleted by doc 18 §3.1 in favour of `<-`, which is implemented in statement position. `<-` in condition position and `guard` onto let-else are outstanding. **Done 2026-11-09**, planned as `docs/24 - 22 - Draw Arrow in Condition Position.md`. Shipped: `<-` in an `if`, a `while` and a `case` arm guard, let-chains laid out as rustfmt lays them out, and `guard` onto let-else. `matches!` left the row — `<-` always binds, so nothing emits it. `if_let_guard`'s stability was **verified** rather than assumed (plan §1.5). Also **`Pattern::Apply`** — `Some v` — which no row owned and without which none of this can be written (plan §10.1); `until` with a pattern is declined as `RDX0107`. Two rustfmt divergences are disclosed in plan §10.6 and belong to row 34 | — | 21 | `done` |
| 23 | `strings-and-literals` | Interpolation with inline format specs; raw, char, byte and C-string literals; `"""` dedent; `##`/`##!` doc comments. **Done 2026-11-09**, planned as `docs/25 - 23 - Strings and Literals.md`. Interpolation with Rust's format specs pasted verbatim, the four literal kinds, and the `"""` block's dedent. The row's doc comments were already plan 17's; their markers are doc 18 §5.1's `-- |` and `-- !`, and `TriviaKind`'s prose still named the old ones until now. **It fixed a silent defect**: `print "hi #{name}"` emitted `println!("hi #{{name}}")`, valid Rust printing the literal text (plan §10.1). Three of the five defects were in **plan 02's lexer**, one a case its own doc comment claimed worked (plan §10.2) — so §1.1's "the lexer is ready" framing is corrected in place | — | 02, 17 | `done` |
| 24 | `data-declarations` | Structs with block init and trailing spread; field punning; primary constructors; enum struct variants and discriminants; granular visibility; type aliases; unit and tuple structs **Done** as doc 18 §2.4 and doc 03 §9.6: `data`, `newtype`, `type`, record literals with punning and update, and the four visibility markers including per-field. Block initialisers and primary constructors are *deleted* by §2.4, which takes `RDX0052` and `RDX0053` with them | — | 18 | `done` |
| 25 | `traits-and-impls` | Default method bodies, associated types and constants, supertraits, generic impls with bounds, `impl Trait` in argument and return position **Done** as doc 18 §2.5: `class` is `trait` and `instance` is `impl`, with default method bodies, associated types and constants, supertraits and generic impls with bounds. `impl Trait` in either position needs nothing — a type is verbatim text (doc 03 §14.1) | — | 24 | `done` |
| 26 | `generics-and-where` | Generic parameters and bounds; `where` as a bound clause in signature position, disambiguated from `where` locals by position **Done** as doc 18 §§2.2 and 5.3: the generic list is synthesised from the types and the bounds are read off `=>`, so neither is written. `where` keeps one role — the position disambiguation this row describes is deleted rather than implemented | — | 25 | `done` |

## Milestone 6 — Full surface (doc 03 Tier C) — **v1.0**

Without these Redox is a syntax skin; with them it is the language docs 01–03 describe.
v1.0 is reached when every row from 01 to 33 is `done`.

| # | Plan | Scope | From Emerald | Depends on | Status |
|---|---|---|---|---|---|
| 27 | `pipeline-and-sections` | `\|>` as a receiver pipeline; operator sections with the auto-ref lowering for comparisons; `(-1)` versus `(- 1)` by adjacency **Done** as doc 18 §4: the pipeline, and all six sections including the method form for comparisons. `(-1)`-versus-`(- 1)` is *deleted* — §4 makes a leading `-` negation always, and the same rule covers `&`, `*` and `!` | — | 19 | `done` |
| 28 | `option-result-operators` | `??`, `??=`, `catch`, `?.` via `?`-in-a-closure, `try` blocks. No implicit clone **Partial.** `??` lowers to a `match` (doc 03 §12.1) and `?` passes through; `??=` is *deleted* by doc 18 §7 in favour of its expanded form. **Done 2026-11-09**, planned as `docs/26 - 28 - Catch, Optional Chaining and Try Blocks.md`. `catch` as a `BinaryOp` at `??`'s level with its binder read from a lambda, `?.` as doc 02 §6.3's closure, and `try` as an immediately-invoked closure — the one row that makes an **unavailable** Rust construct available. It also **corrected doc 03 §12.3's published lowering**, which does not compile and fails doc 03 §2's own first admission test: `.as_ref()` on the final link is type-dependent, so the link takes no method and the path takes a `&` (plan §10.1) | — | 22 | `done` |
| 29 | `comprehensions-and-collections` | Bracket comprehensions and collection-`if`/`for`; literal-shape collect defaults with `into` override; spread; `%w`; set literal; array repeat **Partial.** Doc 18 §3.2's comprehension and the list literal are implemented, and the ascribed-literal lowering `C::from([…])` covers the set and map literals this row names — both the collection-`if`/`for` level and the `into` override are *deleted* by §3.2. **`%w` is deleted, not outstanding** — doc 18 §7's table replaces it with `[…] :: C<_>` and its closing paragraph lists `%w[` among the operators that *"leave outright"*. This cell said otherwise until **2026-11-09**; correcting it is doc 04's own rule 2, and implementing a deleted construct because a stale cell asked for it is the failure mode that rule exists to prevent. So the row had **one** item left, array repeat — **done 2026-11-09**, planned as `docs/27 - 29 - Array Repeat, and Two Retired Tokens.md`, which also retired the dead `PercentW` and `HashBrace` tokens and **corrected doc 03 §10.4's first row**: a bare bracketed literal has emitted an *array* and not `vec![…]` since doc 18 §7 made `[…] :: C<_>` the collection form, and the table never said so (plan §10.1) | — | 19, 21 | `done` |
| 30 | `with-and-use` | `with` dot-shorthand mutation block, subject evaluated once; `use` callback flattening with `RDX0044` on a `return` in the tail **Partial.** `use` is `<-` and its callback flattening is implemented (doc 18 §3.1); `with` is *deleted* by §7. **Done 2026-11-09**, planned as `docs/28 - 30 - Return in a Callback Tail.md`. `RDX0044` — doc 03 §17's own number, the **first** code to use the band §17 reserved for plans 19–28. Doc 03 §7.6's *"disclosed trap, rejected rather than documented away"*: a `return` in a draw arrow's tail leaves the generated closure and the result is valid Rust meaning something else. `?` keeps working, which is the asymmetry the construct exists for (plan §1.3). It also **corrected a test that enforced the reserved band by forbidding it** — as written it would have failed on every plan 19–28 surface code (plan §10.1) | — | 19 | `done` |
| 31 | `functions-advanced` | Multi-clause definitions merged to one `fn` plus `match`; labelled arguments erased in declaration order; `where` locals; `...` forwarding; `&:` method references **Done** as doc 18 §2.3 and §2.2: multi-clause definitions merge to one `fn` plus a `match`, and `where` locals lower to `let` bindings before the body. Labelled arguments, `...` forwarding and `&:` method references are all *deleted* by §7 — the last replaced by `(|> name)`, a section derived from the rule | — | 26 | `done` |
| 32 | `broadcasting` | `f.(xs)` and `.+`-family elementwise operators, borrow marker mandatory, result is an iterator **Superseded.** Doc 18 §7 retires `.(` and the `.+`-family outright, so there is nothing left to broadcast over | — | 29 | `superseded` |
| 33 | `destructuring-and-splat` | Tuple, slice and struct destructuring; rest patterns; splat in argument position. **Plan authored 2026-11-09** as `docs/29 - 33 - Destructuring.md`. **Splat is deleted** — doc 18 §7 removes `...` from the language — and so are doc 03 §6.3's anonymous record patterns with `RDX0042` and `RDX0043` (§2.3, guards do that work). A rest pattern becomes Rust's own `..` and `rest @ ..`, since both tokens are already Redox's: one fewer token and one fewer lowering, the third time doc 18's deletions paid for themselves (plan §1.3). A record pattern takes `=` to match §2.4's literal (plan §1.2). **Done 2026-11-09**, and it also shipped **`Expr::Tuple`** — no row owned a tuple *expression*, so a tuple pattern could only be fed by a Rust function that already returned one (plan §10.1) | — | 21, 29 | `done` |

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
| 34 | `formatter` | `redox-fmt`: trivia-preserving, canonicalizing per doc 03 §16, and therefore the migration tool. Doubles as the answer to Emerald's 113-missed-fixture cutover. **Plan authored 2026-11-09** as `docs/33 - 34 - Formatter.md`, which found that **doc 03 §16's canonical table is stale in eleven of thirteen rows** — doc 18 and doc 30 deleted the alternatives it normalised away — so the plan's §3 rewrites it for the current surface. Every stale row is one where an alternative was *removed*, which means **the formatter got smaller while nobody was looking**, and §16 was the only place that would have said so (plan §1.1). §1.2 states plainly that this is the largest row left and will be `partial` for a while; §8 records why the hazard that sank Emerald's formatter is already solved here, in the emitter's `receiver`. **Partial as of 2026-11-09**: leaves 1 and 5 shipped — `redox fmt` and `--check` work, file-level structure is canonical, every item passes through verbatim by span slice, and §1.4's four properties hold over every `.sdx` in the repository. **Leaf 3 shipped 2026-11-09** as `expr_text`: the single-line expression printer, with §3's rows for juxtaposition, redundant parentheses, method spacing, `::` ascription and trailing commas. It **retired three of §3's own rows**. `f(x, y)` → `f x y` changes the program — `f (x, y)` applies `f` to a **tuple**, which is what juxtaposition being the call form means, and the row outlived doc 03 §16's comma call form (§10.10). The two pipeline rewrites are a **refactor, not a normalisation**: they replace one construct with another, so §1.4's property cannot guard them without reimplementing the emitter inside the test — and a formatter whose riskiest transformation its safety property cannot see is worse than one without it. Offered to plan 38's `RDX1000+` lint band instead (§10.11). Dropping a parenthesis made the precedence table a **second** reader's, so it moved from the parser to `BinaryOp::power` — a copy would drop a needed parenthesis as soon as the two drifted, silently (§10.13). Two defects, both found by a property and neither by the fixture (§10.14). §4's leaf **boundary** is also wrong, not just its order: `if`, `case` and `do` are `Expr` variants whose bodies are statement sequences, so the real division is single-line expressions / block-bearing constructs / items (§10.15). **Leaf 4's pattern half shipped 2026-11-09** as `pattern_text`. It split off cleanly because patterns need neither of the things that make the rest of leaf 4 hard — a pattern is one line, so there is no indentation to re-derive, and a comment in one belongs to the enclosing statement, so there is no trivia to place (§10.17). `Pattern::rust_text` was the wrong thing to reuse in exactly three of eleven forms, each a token Redox spends elsewhere: `Some v` not `Some(v)` because parentheses are grouping, `=` not `:` because `::` ascribes, `or` not `|` because `|` separates a guard ladder (§10.18). And *parenthesise unless atomic* turned out to be **three** rules in pattern position rather than one: after `@` and after `or` the next token is a keyword or an arrow, which no pattern can absorb, so `whole @ Rect { w, h }` and `Some 1 or None` need no parentheses — how much a form needs protecting depends on what comes after it, not on what it is (§10.19). **Leaf 4 completed 2026-11-09**: `stmt_text` prints all fourteen statement kinds with their trivia, `block_text` re-indents at two spaces per level (§3 row 11), and the five block-bearing expressions are printed rather than sliced — no expression form reads the source any more. It corrected §3 row 8, whose example named the one of three `::`-looking forms the rule does not apply to: a local binding takes **one** colon and `n :: u64 = 0` does not parse (§10.22). `else if` has to be reconstructed from a nested block, and a comment on the inner `if` must stop it, since `else if -- why` is not a line Redox can write and dropping the comment is doc 02 §3.2's defect (§10.23). The argument predicate was wrong twice and the corpus caught both: it allowed a list, where `[` after a name is indexing, and it asked about the receiver rather than the printed text, which was not idempotent — **the question is what the printer emits, not what the tree holds** (§10.24). A method section is `(|> len)` and `examples/section.sdx` says so in a *comment*, the second time the corpus's prose carried a spelling the AST could not yield (§10.25). And `if c then a else b` re-parses only as an equation **tail**, which is a constraint leaf 2 must honour: a single-expression body stays on the equation line, or the file stops parsing as Redox and rustc never sees it (§10.26). **Leaves 2 and 6 completed 2026-11-09, so the row is `done`.** The fifteen item kinds print from the tree, `format` is a loop over them, and `redox fmt --check` passes on all twenty-eight examples. Leaf 1's three span models are deleted rather than refactored: **a blank line is a trivium**, so the gaps it measured out of the source were in the tree all along (§10.28). The row's main finding is that **the AST does not record what the author wrote, in three places** — a signature's type variables, uppercased in place by `generalise`; a block string, stored cooked; and a byte string's hex escapes. Each would have silently rewritten working programs, and each was caught by running the formatter over the corpus. Literals are now sliced by span and a **signature line is sliced verbatim**, which is criterion 7 and means the formatter can never normalise a signature's spacing. §10.29 names the fix for a successor: `Signature` keeping the source's names and the emitter applying `rust_generic` when it writes, which is how `TypeApp` already works and the reason an `instance` can be printed while a signature cannot. Two layouts the tree cannot distinguish — an equation's inline body, a single-constructor `data` — are decided by **span**, because where §3 has no row the author's choice is the answer (§10.30). And the plan found the same defect **five times**: one predicate serving two positions. In a language where juxtaposition is application, *does this need parentheses* has a different answer in every position, and the one that mattered changed a program (§10.31). §3 gained five rows it never had, each settled by running the formatter rather than by argument. Eight examples and one golden were reconciled, the golden's diff being three lines of redundant parentheses in the generated Rust — which is the proof the changes preserved meaning (§10.32). The properties caught three bugs no reading would have (plan §10.2), and one of them inverted the plan's own blank-line rule (§10.3) | `crates/emerald-fmt`'s goal and its approach's disclosed failure — a pure AST pretty-printer loses every comment | 17, 19 | `done` |
| 35 | `test-blocks` | `test "…"` to `#[cfg(test)]` module with `#[test]` fns; deterministic name slugs; `assert` family **Done** as doc 03 §15: `test "…"` becomes one `#[cfg(test)]` module per file with deterministic name slugs and an index on collision, and `assert` reads the operator the author wrote. The run corpus compiles the generated module as a test binary and runs it | `crates/emerald-cli/tests/examples.rs` compile-and-run-and-assert-stdout harness | 23 | `done` |
| 36 | `typed-domain-errors` | `errors` declaration generating the enum plus `Display`, `Error` and `From` impls. `std` only **Done** as doc 03 §12.5: the enum, `Display`, `Error` and a `From` per `from` row, naming nothing outside `std` | — | 28 | `done` |
| 37 | `defer` | Scope-exit blocks, LIFO, via a generated `core`-only `Drop` guard. **Blocked on doc 03 §23 item 1** — the one disclosed exception to doc 01 §36 **Superseded.** Doc 18 §3.1 deletes `defer`, which closes doc 03 §23 item 1 — the one disclosed exception to doc 01 §36 — by removing the construct that needed it | — | 20 | `superseded` |
| 38 | `panic-and-lints` | `todo`/`unimplemented`/`unreachable`/`panic`/`assert` spellings; `RDX1001` `unwrap` discouragement **Partial.** `todo`, `unimplemented`, `unreachable`, `panic` and the `assert` family are implemented (doc 03 §12.6); `RDX1001`'s `unwrap` discouragement is outstanding — **plan authored 2026-11-09** as `docs/32 - 38 - Unwrap Discouragement.md`, and it is the row's last item. Doc 03 §12.7 says the lint *"costs nothing because no analysis is involved"*, which is true of the detection and false of everything around it: this is the **first `RDX1000`-range code and the first diagnostic that is not an error**, so a severity, a non-fatal path through `check`, and a suppression spelling all have to exist first (plan §1.1). Suppression is a comment pragma, because an attribute either breaks rustc or breaks plan 18's attributes-are-trivia rule (plan §1.3). **Done 2026-11-09.** It needed a new **`tests/lint/`** corpus, because `tests/ui/` asserts every case *fails* to build and that assertion is load-bearing (plan §10.1); `plain_one` turned out to hardcode the word `error` (§10.2); and the lint's first contact with real code caught four call sites in `examples/` that the corpus had not been checking for, since it demanded zero *rustc* output and said nothing about Redox's (§10.3) | — | 23 | `done` |
| 39 | `async` | `async` functions and blocks, postfix `.await`, no executor. **Postfix `.await` is gone** — `docs/30 - Bind.md` makes await `x <- f`, already shipped — so this row is the `async` marker on a signature, `async` blocks, and the example the await-bind does not yet have. **Doc 30 §8.4 is a debt against this row**: `<-` is proved by eight emit tests and not by a running program, because every await needs an `async` function and `run :: async ()` currently emits `fn run<ASYNC>() -> ASYNC ()` — the marker read as a type variable. `examples/async.sdx` is this row's first deliverable. **Plan authored 2026-11-09** as `docs/31 - 39 - Async.md`: two markers and one example. Its §1.4 answers the question no previous row faced — how a corpus with **no runtime** runs an async program — by having the example write a nine-line `block_on`, verified to compile before the plan was written. That is doc 03 §14.3's *"the runtime is the user's"* demonstrated rather than asserted. **Done 2026-11-09**, and **doc 30 §8.4's debt is paid** — `<-` is now proved by a program that runs. The example compiled and ran correctly on the first attempt, a first in this run, because §1.4's executor was verified before the plan was written (plan §10.1) | — | 26 | `done` |
| 40 | `lsp` | `redox-lsp` on `lsp-server` + `lsp-types` (doc 02 §5.5): diagnostics on change, semantic tokens, document symbols, and a formatter hook onto plan 34. Rustc errors arrive already remapped to the `.sdx` through plan 07, so the server reports them against the file the user wrote. Editor-agnostic by construction — plan 49 ships the clients. Emerald does this in 545 lines; Redox's is simpler, having no symbol table to build. **Plan authored 2026-11-09** as `docs/35 - 40 - LSP.md`, which verified that `lsp-server` 0.10 and `lsp-types` 0.97 resolve and that every input the four capabilities need is already public — **except one**. This cell's claim that *rustc errors arrive already remapped* is not true yet: plan 07's remap is `pub(crate)` inside the CLI's `check` command, so leaf 1 moves it to `redox-driver` before any LSP code exists. That is plan 34 §10.13's argument one row later — a `pub(crate)` table is right until a second reader appears, and a *copy* would aim an editor's squiggle at the wrong line while still compiling (plan §1.2). The rustc pass itself is **out of scope** and §6 says why: cancellation and debounce are a larger design than the rest of the row, while the Redox-side diagnostics are the ones that fire between keystrokes. Completion, hover and go-to-definition are declined outright — all three need name resolution, which AGENTS.md forbids Redox from having, so each would be a guess. **Shipped 2026-11-09**: `crates/redox-lsp`, a library with a thin binary and nine tests over an in-process `Connection`. Leaf 1 moved the remap as its own commit. The findings are about the **protocol's** shape rather than Redox's: `lsp-types` 0.97 has `Uri` and not `Url`, and `lsp-server` 0.10's `Response` carries one `response_result` rather than a `result`/`error` pair — so §1.1's *measured, not assumed* was satisfied by a command that measured versions and not names (§10.1). An **empty** `DocumentSymbolResponse` is ambiguous on the wire, both variants being JSON arrays, so the test asserts `json!([])` rather than a Rust variant (§10.2). And criterion 7's broken document turned out to break at the *lexer*: unbalanced delimiters are `RDX0012`, a lexical code, because the lexer tracks bracket depth — so the test is now two, and only the parse-failure one proves that **colour survives a syntax error** (§10.3). Two legend entries are deliberately unreachable, `FUNCTION` for want of name resolution and `COMMENT` because a comment is trivia and never a token (§10.5). Moving the document store under `serve/` to satisfy doc 06 §2.1 made it `pub(crate)` and immediately exposed two dead accessors written for tests that did not exist (§10.4) | `crates/emerald-lsp` (545 lines, directly adaptable) and plan 90's warning to purge `(`-assuming heuristics first | 07, 34 | `done` |
| 41 | `tree-sitter-grammar` | `tree-sitter-redox/` per doc 02 §5.4: `grammar.js` plus an external scanner for INDENT/DEDENT, the generated parser, `highlights`/`injections`/`locals` queries, and a corpus suite — which doubles as an independent check on the grammar's shape (doc 02 §9.4). The cost is disclosed, not hidden: an indentation-sensitive tree-sitter grammar is genuinely fiddly, and it is a second grammar to keep in sync with plan 46's inventory | Emerald plan 24's `tree-sitter-emerald` layout and corpus-test convention | 19 | `done` — **scope narrowed 2026-11-08**: the editor configs this row carried moved to plan 49, which names Vim, Neovim, VSCode and Cursor explicitly and is too large to ride along with a grammar. **Plan authored 2026-11-09** as `docs/36 - 41 - Tree-sitter Grammar.md`, which spiked the toolchain end to end first because every claim in it is about a toolchain rather than about Redox: `tree-sitter` 0.26.11, an external C scanner holding a stack of indent widths, `generate`, `parse` and `test` all verified working. It found two gaps the register did not name — a grammar with no `tree-sitter.json` is silently generated against the **older ABI 14**, and `tree-sitter` is on the *system* `PATH` rather than pinned by `devenv.nix`, so a committed parser could not be regenerated identically elsewhere. Leaf 1 closes both. The row's shaping decision is §1.2: the grammar aims at **highlighting, folding and selection — not parity** with `redox-parser`. The compiler is the authority on what parses, and a second one would make every disagreement a bug report against the wrong component; so the grammar is precise about the off-side rule and deliberately flat inside an expression, which also avoids a second precedence table of the kind plan 34 §10.13 found drifts silently. **This is a constraint on row 46**: its inventory must compare token kinds and block structure rather than productions. And §1.3 makes the main assertion a sweep over every `.sdx` for a parse with no `ERROR` node, rather than hand-written s-expressions — those check what the author last thought, the corpus checks the language. **Shipped 2026-11-09**: the grammar, a nesting-aware external scanner, the ABI 15 parser, three queries, five corpus cases and a Rust sweep in the gate. All twenty-eight examples and both lint cases parse with **no `ERROR` node**, `lexical.sdx` included, and an inconsistent dedent does error. The row's difficulty was entirely in the scanner, and it took **three designs**: a scanner cannot emit NEWLINE then INDENT at one place, because `advance(…, true)` consumes, so the third design is tree-sitter-python's — **INDENT absorbs the newline** — plus two `pending` fields for the tokens the grammar is not ready for, since the width cannot be measured twice (§10.1, §10.2). The other costly lesson is that **`prec` is static and `conflicts` is dynamic**: `section` at `prec(2)` discarded the reading the parser needed and broke every nested parenthesis followed by a postfix (§10.3). §1.2's flat expression rule held, and its **edges** were not free — separators had to leave the operator token, a comma had been riding in it, and an ascription had to become a rule (§10.4). A pattern is a **run** in four places, which is plan 34 §10.19's finding met from the other side (§10.6). And §1.3 was right within minutes: four of the five hand-written corpus expectations were wrong about the tree (§10.8). Criterion 5 is **half-met** and says so — nothing yet asserts that `highlights.scm`'s coverage equals plan 40's semantic-token table (§10.9) |
| 42 | `fuzzing` | `lex_source`, `parse_program`, `emit_program` and a generated-program target whose oracle is *the generated Rust must parse* — a stronger oracle than Emerald had | `crates/emerald-fuzz-smoke` architecture: logic in a stable crate, `fuzz_targets/*.rs` as three-line wrappers, zero dependencies | 04 | `not-started` |
| 43 | `test-suite-expansion` | Property tests: lexer indentation round-trip, formatter idempotence, transpile stability, `parse(fmt(x)) ≡ parse(x)` modulo trivia. **Shipped 2026-11-09** as `docs/34 - 43 - Test Suite Expansion.md`, which found **three of the four already existed**: transpile stability is plan 04's `emission_is_deterministic`, and both formatter properties are plan 34 leaf 5's — each built by the row whose claim could not be proved without it, which is the healthy direction (plan §1.1). So the row delivered **one** property, the indentation round-trip in `redox-lexer/tests/indentation_round_trip.rs`, which is also plan 34 leaf 4's safety net: that leaf re-indents from the same `Indent`/`Dedent` tokens, so if they disagree with the text the formatter changes block structure silently (plan §1.3). It asserts the form the width stack actually claims — one token depth is one text width within a file — since Redox has no fixed indent unit to multiply (plan §10.2). The stated dependency on row 42 was **not real** and the direction is corrected: none of the four needs a fuzzer, and a fuzzer would point *at* these assertions (plan §1.2) | — | 34 | `done` |
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
