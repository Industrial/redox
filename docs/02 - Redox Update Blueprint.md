# Redox — Update Blueprint, Derived from Emerald

Version: 0.1.0
Status: proposal / review draft
Supersedes: nothing. Amends `docs/01 - Redox.md` (the base specification) by addition and by
named correction.
Basis: a direct source audit of `/data/Code/emerald` (2026-11-08) against `docs/01 - Redox.md`.

---

## 0. How to read this document

`docs/01 - Redox.md` is the language specification. This document does not replace it. It does
three separate things, kept separate on purpose:

1. **§§1–3 — What Emerald actually is, and what transfers.** Emerald is a 107k-line,
   203-plan, LLVM-backed native language. Redox is a source-to-source translator. Most of
   Emerald's mass does not transfer. Saying precisely which parts do, and which parts are
   actively the wrong lesson, is most of this document's value.
2. **§§4–9 — How to build the Redox compiler.** Crate split, toolchain decisions, the
   lowering discipline, source maps, diagnostics, incrementality, testing ladder, and the
   editor/tooling surface. This is the constructive part.
3. **§§10–13 — Specific defects and gaps in `docs/01 - Redox.md`,** each with a proposed
   resolution, plus the revised roadmap and the decisions that need a human answer.

Everything cited from Emerald carries a real path. Where a claim is Emerald's own
self-report rather than something verified here, it says so.

---

## 1. What Emerald is, measured

Verified directly from the repository, not from its README's claims:

| Property | Value | Source |
|---|---|---|
| Workspace crates | 10 members + a fuzz-smoke crate | `Cargo.toml` `[workspace] members` |
| Largest single file | `crates/emerald-codegen/src/lib.rs`, **34,429 lines** | `wc -l` |
| Second largest | `crates/emerald-sema/src/lib.rs`, **22,589 lines** | `wc -l` |
| Lexer | `crates/emerald-lexer/src/lib.rs`, **116 lines** (`logos`) | `wc -l`, `Cargo.toml` |
| Parser | `lalrpop` 0.22 + a 4,723-line `lib.rs` + 1,497-line `ast.rs` | `crates/emerald-parser/` |
| Runtime/stdlib | `crates/emerald-rt`, **~58k lines across ~60 modules** | `wc -l` |
| Examples | **135** `.em` files, all CI-compiled-and-run | `examples/`, `crates/emerald-cli/tests/examples.rs` |
| Plan records | **203** timestamped plans in `history/` | `ls history/` |
| Self-reported | 107k lines Rust, 1,353 tests passing, 0 clippy warnings | `README.md` (not re-verified here) |

Emerald's shape, in one line: **Ruby's surface syntax recompiled onto a fully static type
system, AOT-compiled via LLVM, with an Erlang-style actor runtime** — and no GC and no
ownership system, disclosed as a permanent choice rather than a gap (`spec/RUNTIME.md` §1).

### 1.1 The single most important number

**Of 203 plans, plans 92–203 — 112 of them, ~55% of the entire project budget — are
standard-library construction.** Read the list: `raw-tcp-udp-sockets`, `dns-resolution`,
`tls`, `http-client`, `http-server`, `websocket`, `grpc`, `quic-http3`, `crypto-hashing`,
`crypto-aead`, `jwt`, `x509-certificates`, `json`, `toml`, `yaml`, `csv`, `regex`, `xml`,
`protobuf`, `gzip`, `zstd`, `tar`, `zip`, `lz4`, `brotli`, `sqlite`, `postgresql`, `mysql`,
`redis`, `s3`, `image-codec`, `pdf-generation`, `audio-decode`, `smtp-email`,
`terminal-ui`… 112 plans, culminating in a ~58k-line, multi-hundred-megabyte static
archive (`crates/emerald-rt`, and see §3.4 on what that archive cost operationally).

**Redox gets all of that for free, permanently, by construction.** Every crate on
crates.io is a Redox library the moment the transpiler emits valid Rust. This is not a
minor convenience — it is the structural reason Redox can be a small project where Emerald
could not be. `docs/01 - Redox.md` §1.2 already states "No Redox standard library" and §26 "Max
Rust ecosystem interoperability"; what Emerald's history supplies is the *evidence of the
stakes*. Those two lines are worth more than every other line in the spec combined.

**Rule to adopt verbatim:** no Redox plan ever implements a library function. Not one. If
a feature's implementation would add a runtime symbol, a prelude item, or a bundled crate,
it is misdesigned. The `print` → `println!` mapping in `docs/01 - Redox.md` §3.3 is the only
sanctioned exception, and §10.9 below argues even that one should be narrowed.

---

## 2. What transfers from Emerald, and what does not

The asymmetry is the whole analysis. Emerald owns semantics; Redox borrows them. Emerald
must answer "what does this mean?"; Redox must answer only "what Rust do I write?"

| Emerald component | Lines | Transfers to Redox? |
|---|---|---|
| `emerald-lexer` (logos) | 116 | **Partially.** Redox needs a hand-written off-side-rule pre-lexer instead — see §5.1. |
| `emerald-parser` (LALRPOP) | 7,372 | **No — as a technology choice.** See §5.2: LALRPOP is the wrong tool for Redox. The *methodology* transfers fully. |
| `emerald-sema` | 22,589 | **Almost entirely no.** This is the type checker, borrow checker, purity checker, contract checker. Redox has rustc. See §6. |
| `emerald-codegen` (LLVM/inkwell) | 34,429 | **No.** Replaced by a few thousand lines of Rust source emission. |
| `emerald-rt` (stdlib) | 57,972 | **No.** See §1.1. |
| `emerald-driver` (cache, parallel, require graph) | 2,658 | **Yes, nearly verbatim.** See §7. The best-value transfer in the repo. |
| `emerald-fmt` | 1,670 | **Yes as a goal, no as an implementation.** Its comment-loss defect is fatal for Redox. See §3.2. |
| `emerald-lsp` | 545 | **Yes.** Small, high leverage. See §9. |
| `emerald-mcp` | 392 | **Yes, optionally.** See §9. |
| `emerald-fuzz-smoke` + `fuzz/` | 1,861 | **Yes.** Zero-dependency design is directly reusable. See §8.5. |
| `history/` plan-of-plans discipline | 203 files | **Yes.** See §3.1. |
| `examples/` as CI-checked truth | 135 files | **Yes — the highest-value process transfer.** See §8.3. |
| `spec/GRAMMAR.md` graded inventory | 761 lines | **Yes — the highest-value *design* transfer.** See §4.1. |

Net: of Emerald's ~107k lines, the Redox-relevant surface is roughly **`emerald-driver` +
`emerald-fmt` + `emerald-lsp` + the fuzz harness ≈ 6.5k lines**, plus the process and
grammar-design methodology, which is free. A Redox that stays near 8–12k lines of Rust is
consistent with everything Emerald's history shows about where lines actually go.

### 2.1 The inverted lesson

Emerald's inception §2.2 reads: *"Do not turn Emerald into Rust with Ruby syntax. Rust
concepts should not be imported merely because they are fashionable or useful."*

Redox is the exact opposite project: it **is** Rust with MoonScript syntax, by design and
by charter. So this principle inverts rather than transfers, and the inverted form is a
real constraint worth writing down:

> **Do not turn Redox into a language.** No Redox concept may exist that is not a
> mechanical restatement of a Rust concept. If a Redox feature needs the transpiler to
> *know* something Rust doesn't tell it, the feature is wrong, not the transpiler.

Emerald's own history is the proof of what happens otherwise. It added `pure`,
`comptime`, `requires`/`ensures` contracts, `own`/`borrow`/`borrow var`, actors,
supervision trees, and a consistent-hash distribution layer — each individually
defensible, and collectively the reason `emerald-sema` is 22,589 lines. Every one of those
required the compiler to own a semantic judgment. Redox must own zero.

---

## 3. Process and culture lessons

These cost nothing to adopt and are, in aggregate, probably worth more than the technical
content below.

### 3.1 Plan-of-plans with honest status

`history/2026-09-08T174011Z-plan-of-plans.md` is an index of every plan with
`not-started | planned | in-progress | done`, and it explicitly refuses to hold plan
content: *"This document is an index only. It does not author plan content."*

What makes it genuinely good rather than ordinary project tracking is the **stale-label
correction convention**. Dozens of rows read like this:

> `done` — stale label, corrected 2026-09-21: `goto_definition`/`completion_items`/
> `semantic_tokens_legend` all real in `emerald-lsp`

and some admit the opposite:

> row 42 | `superseded` — corrected 2026-09-27: this row's own real scope was finished by
> row 70, which exists precisely because this row's first attempt was incomplete

Rows are **never deleted when superseded** — plan 43 (`nullable-types-and-safe-navigation`)
is kept with a banner saying plan 73 replaced it outright. The record of what was tried
and abandoned is treated as more valuable than a tidy list.

**Adopt:** `docs/04 - Redox Plan of Plans.md` as an index-only table;
`docs/05 - NN - <Name>.md` per plan; correct stale labels in place with a date and the
evidence; never delete a superseded row.

### 3.2 Comment preservation — Emerald's defect, Redox's hard requirement

`crates/emerald-fmt/src/lib.rs`'s own module doc discloses this cleanly:

> **Disclosed limitation: comments are lost.** `emerald_parser`'s lexer treats an ordinary
> `#`-comment as pure whitespace and it has NO AST representation at all — parsing and
> re-emitting a file WILL drop every such comment.

Only `##` doc-comments attached to one of six declaration kinds survive, as a
`doc: Option<String>` AST field.

For Emerald this is a formatter annoyance. **For Redox it would be a fatal product
defect**, and `docs/01 - Redox.md` currently specifies the defect directly:

> §4.1: "Comments are removed during compilation."

That cannot stand. The generated `.xrs` is a **human-facing artifact** — §3.4 of the spec
says "Output must be deterministic and readable," §29 says Rust errors are passed through,
and §1.1 shows `.xrs` being handed to rustc. A reader debugging generated Rust with every
comment stripped is reading a decompilation. Worse: stripping comments destroys
`// SAFETY:` justifications, `#[allow]`-adjacent explanations, and license headers.

**Resolution (spec delta, §10.1):** comments are **trivia attached to AST nodes**, not
whitespace. The lexer emits them; every AST node carries `leading: Vec<Trivia>` and
`trailing: Option<Trivia>`; the emitter re-emits them. `#` → `//`, `###…###` → `/* … */`,
and a doc-comment form maps to `///` / `//!`. Decide the doc-comment spelling now (§13).

This is also the reason Redox's formatter must be trivia-preserving rather than a pure AST
pretty-printer — see §9.2.

### 3.3 Decline permanently, in writing

Emerald separates *deferred* from *declined*. `spec/SPEC.md`:

> No `eval`, `method_missing`, `send`, monkey-patching, open classes, runtime reflection,
> or duck typing. … **These are not gaps to be filled later**; they are inception §2's own
> scope reduction.

And `spec/GRAMMAR.md` grades each Ruby production `KEEP` / `MODIFY` / `REMOVE` /
`UNDECIDED`, where every `UNDECIDED` **links to the named place it gets resolved** —
"nothing is left open without a named place it gets resolved."

`docs/01 - Redox.md` already leans this way (§13's "Standalone existential checks such as
`value?` are excluded", §17.7's "`extends` is excluded"). Make it systematic: a
`docs/DECLINED.md` with a reason per entry, and `UNDECIDED` entries that name their
resolution venue. §12.4 proposes the initial decline list.

### 3.4 Measured claims, and recording the traps

Emerald's `README.md` ends its status section with *"None of this is rounded up. Where a
claim couldn't be verified directly it isn't made"* — and then discloses that it loses to
Crystal on four of six benchmarks and leaks memory outside a region. `examples/README.md`
tracks *"the specific gaps and bugs found while writing them, rather than only listing
what succeeded."*

Separately, `AGENTS.md` carries a **"Known tooling gotchas"** section recording two
environment traps that each produced a *false compiler-bug report* before the cause was
found: a context tool silently truncating looped subprocess output (which manufactured a
phantom "generic method repeated-print bug", plan 71), and `cargo nextest` at 32 threads
filling a 2 GB `/tmp` with ~112 MB runtime archives per process, failing **39 unrelated
tests together** and reading as a broad compiler regression.

Redox's `AGENTS.md` already has the right instinct here — the blaze section documents that
`blaze doc` panics, that unknown subcommands exit 101, and that rustc remains the final
word. Keep that genre and extend it as traps are found. Add `docs/STATUS.md` with verified
numbers and a visible gap table.

### 3.5 Prototype before committing, and write down the revisit trigger

`spec/COMPILER.md` is the model artifact here. Every toolchain decision was made *by
building a real prototype*, with a comparison table and an explicit revisit condition:

- **LALRPOP vs Chumsky** — both prototyped against the identical grammar slice, a 6-axis
  table, and an honest "the error-message quality was a wash." Chumsky lost partly on a
  real bug hit during the prototype (`text::ascii::ident()` stopping at the first digit, so
  `"Int64"` lexed as `"Int"`).
- **Cranelift vs LLVM** — Cranelift chosen for v1 with a named revisit trigger. **The
  trigger actually fired**, and the record of what happened next is the valuable part:
  Cranelift was *tuned first so the bake-off wouldn't be rigged* (`opt_level=speed`,
  measured, within noise), then LLVM was wired in and measured at 0.762 ms vs Cranelift's
  4.260 ms on `sum`. Then a follow-up plan deleted Cranelift entirely and **ported its
  entire 17-test suite verbatim — same source strings, same expected outputs — as the
  parity proof.**

**Adopt all three habits:** prototype with real code before choosing; tune the incumbent
before the bake-off; when replacing a component, port the old component's test suite
verbatim as the parity proof. §5.2 applies the first one to Redox's parser decision.

### 3.6 Decide big design questions with independent passes

`spec/OWNERSHIP.md` §1 records that before writing an ownership design, **four independent
research passes** studied Rust's borrow checker, Austral's linear types, Vale's regions,
and Pony's reference capabilities — each asked separately, *without seeing the others'
answers*, whether to recommend its own subject. All four said no to their own subject and
converged on a smaller hybrid. The decision then **overrode that convergence deliberately
and on the record**, while using the research to avoid the three specific mechanisms it
identified as costliest (named lifetime parameters, variance, higher-ranked trait bounds).

That is how to run Redox's genuinely open questions (§13) — independent analysis, recorded
convergence, and an explicit owner's decision that may override it, with reasons.

---

## 4. Language lessons from Emerald

### 4.1 Grammar as a graded inventory — adopt immediately

`spec/GRAMMAR.md` is a table over *Ruby's grammar areas*, each tagged `KEEP` / `MODIFY` /
`REMOVE` / `UNDECIDED` with one example per keep, covering literals, identifiers,
assignment, operators and precedence, control expressions, method definitions, calls,
classes/modules, blocks, exceptions, pattern matching, and more. Inception §18 called for
exactly this: *"import Ruby's grammar as an inventory, mark KEEP/MODIFY/REMOVE per
production."* It was even a tiebreaker in the parser choice — a declarative `.lalrpop`
file *is* a production list, which fit the inventory shape better than a combinator chain.

Redox needs the same artifact, but it has **two** source grammars and therefore a richer
grading:

- **MoonScript's grammar** — graded `KEEP` / `MODIFY` / `REMOVE` for what Redox's surface
  adopts.
- **Rust's grammar** — graded `PASSTHROUGH` / `SUGARED` / `UNREACHABLE` for what the
  target language can express. This axis has no Emerald analogue and is where Redox's real
  risk lives: **any Rust construct marked `UNREACHABLE` is a construct Redox users cannot
  write**, which directly violates §26's interoperability charter.

Run that second inventory now, before more syntax is designed. Candidates that an audit of
`docs/01 - Redox.md` suggests are currently `UNREACHABLE` — i.e. real gaps:

| Rust construct | Status in `docs/01 - Redox.md` | Severity |
|---|---|---|
| Attributes (`#[derive]`, `#[cfg]`, `#[tokio::main]`, `#[serde(...)]`) | **absent entirely** | critical — see §10.2 |
| Field/item visibility beyond `export` (`pub(crate)`, `pub(super)`) | absent | high |
| `where` clauses | not mentioned (§5.3 covers inline bounds only) | high |
| `impl Trait` in argument/return position | absent | medium |
| `match` ergonomics: `@` bindings, `|` alternatives, `ref` | §18 covers arms + guards only | medium |
| `loop` / labeled break / `break 'label value` | absent (§8 has while/until/for) | medium |
| `const` / `static` items, `const fn` | §4.3 implies const but see §10.5 | high |
| `type` aliases, `union`, `extern` blocks, `mod` blocks | absent | medium |
| Closures with `move`, explicit return types | §5.4 covers `->`/`=>` only | medium |
| Raw strings, byte strings, char literals | absent (§7 covers interpolation only) | medium |
| Tuple structs, unit structs, `#[non_exhaustive]` | §17.1 covers braced structs only | medium |

The spec's §23.1 inline-Rust escape hatch is the designated relief valve for all of these,
and it is the right instinct. But **an escape hatch used for attributes and `where` clauses
is not an escape hatch, it's the main road.** Grade the inventory, then promote the
high-frequency items to first-class syntax.

### 4.2 Immutable by default — already right, and independently confirmed

Emerald's plan 72 (`immutable-by-default-bindings`) made bindings immutable unless declared
`var`, as part of its "Sable alignment" re-inception. `docs/01 - Redox.md` §4.3 already does
this (`name = "Alice"` vs `mut counter = 0`), with the advantage of mapping 1:1 onto
`let` / `let mut`. Nothing to change; worth noting that Emerald arrived at the same answer
from a dynamic-language starting point, which is mild evidence it's the right default.

Note one asymmetry Emerald had to spell out and Redox should too: `var` applies **to plain
locals only** — fields and parameters follow separate rules (`spec/GRAMMAR.md` §2). Redox
needs the equivalent statement for `mut` on parameters (`mut x: T` patterns), fields, and
`for` bindings.

### 4.3 `nil` → `Option[T]`: the lesson is *don't flatten*, and *respell consistently*

Emerald's plan 73 removed `nil`, `Nil`, and `T?` **outright** in favour of
`Option[T]`/`Some`/`None`, and that removal cascaded: `x ||= expr` was deleted with no
replacement assignment form, Ruby's `&.` was respelled `?.` and restricted to
`Option[T]`-typed receivers only, and `??` was added as the designated default-providing
operator. `spec/GRAMMAR.md` §§3–4 record each of those as a linked supersession rather than
a silent edit.

Redox starts from Rust's `Option`, so the destination is already correct (§13). Two
transferable specifics:

1. **"Do not silently flatten nested options"** (`docs/01 - Redox.md` §13.1) matches Emerald's
   rule that `?.` wraps the result back into `Option[U]` rather than collapsing. Keep it,
   and see §6.3 for a lowering that gets it right *without* a type checker.
2. **Respell consistently or not at all.** Emerald paid for having `&.` in the spec, then
   `?.` in the grammar, then stale `&.` references needing a correction pass
   (`GRAMMAR.md`'s own status header lists `&.`→`?.` among its stale-syntax fixes). Redox
   uses `?.` and `??`; make sure §13's `?.` never collides with Rust's `?` operator in the
   lexer (`x?.y` vs `x?` followed by `.y` — a real one-token-lookahead decision that must
   be written down).

### 4.4 Reconsider `@@`

Emerald **removed** `@@` deliberately (`spec/GRAMMAR.md` §2):

> Class variable identifiers (`@@foo`) | **REMOVE** | Ruby's `@@` semantics (shared,
> inheritance-crossing, easy to misuse) add complexity inception §22 rule 9 warns against

`docs/01 - Redox.md` §16.2 reuses that exact sigil for an unrelated meaning: `@@create!` →
`Self::create()`. MoonScript's own `@@x` means `self.__class.x` — i.e. the class-variable
reading Emerald threw out. So Redox's `@@` is a familiar sigil carrying a *different*
meaning to exactly the audience most likely to recognise it.

It's defensible — `Self::` is genuinely the static-dispatch analogue — but it is a
deliberate decision, not an obvious one. Flagged for confirmation in §13.

### 4.5 Syntax churn is extremely expensive — freeze early, and own the migration

Emerald's plan 71 (`grammar-unification-fn-and-do-end`) removed `->`, respelled `case`/
`when` → `match`/`do`, and made type annotations mandatory on every local. The cost is
recorded in plan 90's own leaf notes:

> plan 71's own record shows a migration scoped only to `examples/`/`benchmarks/`/
> `spec/*.md` previously **missed 85+28 embedded `.em` fixtures baked into
> `emerald-codegen`/`emerald-driver`/`emerald-cli`/`emerald-mcp`'s own test modules**

113 missed fixtures in one cutover, because test sources were string literals inside Rust
test modules instead of files. Two lessons:

1. **Never embed language source as string literals in Rust tests.** Every `.sdx` fixture
   lives in a file under `examples/` or `tests/`, and tests reference files. This makes a
   syntax migration a `redox fmt --migrate` run over a directory instead of an
   archaeological grep.
2. **Freeze the syntax surface before writing 135 examples,** and when a cutover is truly
   needed, ship the migration as a tool, not as instructions. §9.2's formatter is the
   natural home.

### 4.6 The parenless-call ambiguity — Emerald already did Redox's homework

This is the single most directly applicable piece of Emerald's record, because
`docs/01 - Redox.md` §6.1 specifies exactly the feature Emerald found hardest to parse.

Emerald's plan 07 deferred general parenless calls with a precise diagnosis:

> A general `Ident followed-by-Expr` command-call rule and a general `Expr-as-statement`
> rule are genuinely LALR(1)-ambiguous when both start with a bare `Ident`… Reserving
> `puts` as a literal keyword token — the only builtin this compiler has — sidesteps the
> ambiguity entirely. **Revisit when a second builtin or user-defined command-call syntax
> is needed.**

So for 89 plans, Emerald's `puts x` worked **only because `puts` was a hardcoded grammar
keyword** (`grammar.lalrpop:622-626`), while `Point.new 3.0, 4.0`, `Math.sqrt @x * @x`, and
`greet name: "yo"` were all parse errors. Plan 90 is the revisit, and it lays out the two
real options:

- **(A) Statement-initial position only** — the precedent already in that grammar
  (`grammar.lalrpop:1010-1022`, where leading unary `-`/`!` is forbidden at statement start
  but legal everywhere else). Cheap. **Cost: a nested parenless call stays illegal as
  another call's argument** — `foo(bar x)` still needs `bar`'s parens.
- **(B) Whitespace-adjacency lexing** — Ruby's and Crystal's actual mechanism: emit a
  distinct token depending on whether `(` immediately follows an identifier with no
  intervening space, so `foo(x)` is an ordinary call and `foo x` / `foo (x)` is a command
  call. Fully general. **Cost: a bigger lexer change, and it reintroduces the `foo -bar`
  unary-vs-binary ambiguity for command-call arguments.**

**Recommendation for Redox: (B).** The cost that made (B) unattractive for Emerald does not
apply here — Redox's lexer is **already** hand-written and already column-aware, because
indentation is significant (§5.1). Adjacency information is free in a lexer that is already
tracking whitespace to emit INDENT/DEDENT. Option (A) would also break
`docs/01 - Redox.md` §6.4's own worked example, `result = add (multiply 2, 3), 4`, which is a
nested parenless call in argument position.

Take two further specifics from plan 90 directly:

- **The formatter and the parser must agree, and that is an architectural question, not a
  style nit.** `emerald-fmt` *always re-adds parens* (`lib.rs:1525-1533`) precisely because
  it runs off the bare AST with no type information and cannot tell a legal parenless call
  from an illegal one. So rewriting examples parenless would have them silently reformatted
  back by the same CI gate. Redox must decide its canonical spelling **before** writing
  examples, and the formatter must be able to produce it without type information.
- **Check the LSP for `(`-assuming heuristics** before parenless calls become common
  in-tree. Plan 90 has a dedicated leaf for this.

### 4.7 One way to write it

Emerald's plans 86 and 87 (`block-attached-call-do-end-alternative`, then
`do-end-exclusive-block-attachment`) converged on a single block-attachment syntax, and
`GRAMMAR.md`'s status header records the brace form `{ |x| ... }` being **removed
outright** rather than kept as an alternative.

Redox's indentation already gives it one block form, which is better. But `docs/01 - Redox.md`
currently has multiple spellings for the same thing in several places — `(a, b) = …`
destructuring vs `[first, second] = values`; `->` vs `=>` closures (§5.4, whose "receiver
context" condition is not defined); implicit vs parenthesized calls (§6.1/§6.2, where §6.2
says parens "should be encouraged for ambiguous nested expressions" without saying which
are ambiguous). Pick one canonical form per construct, let the formatter enforce it, and
accept the others as input only.

### 4.8 Self-hosting: reframe it

`docs/01 - Redox.md` §34 and Phase 7 make self-hosting a goal. Emerald never self-hosted and
shows no sign of trying, at 107k lines and 203 plans. That's weak evidence, but the
stronger argument is specific to Redox: **a transpiler's self-hosting is nearly free, and
therefore nearly meaningless as a milestone.** `.xrs` *is* Rust, so "the self-hosted
compiler" is just the Rust bootstrap with its source written in `.sdx` and its generated
`.xrs` committed. There is no chicken-and-egg problem and no bootstrap chain to verify.

**Reframe Phase 7 as a conformance test, not an architecture goal:** rewriting the
compiler in Redox is a *measurement of Redox's expressiveness* — and an excellent one,
since it will find every `UNREACHABLE` Rust construct in §4.1's inventory the hard way.
Commit the generated `.xrs` so the bootstrap never breaks, and keep the Rust implementation
permanently rather than as a "fallback" (§34's own wording). Phrase the acceptance
criterion as "the compiler's source can be expressed in Redox without the inline-Rust
escape hatch," which is the property actually worth proving.

---

## 5. The Redox compiler: toolchain and crate split

### 5.1 Lexer — hand-written, off-side rule, trivia-preserving

Emerald uses `logos` in 116 lines, because Ruby-ish syntax with `end` delimiters needs no
whitespace sensitivity. **Redox cannot use a plain regex-dispatch lexer**, because
indentation is syntactically significant (`docs/01 - Redox.md` §3.1) and because of §4.6's
adjacency requirement.

Build `redox-lexer` by hand. Responsibilities:

1. **Off-side rule.** Maintain an indent stack; emit `Indent` / `Dedent` / `Newline`
   tokens. Four spaces per level; reject tabs or normalize by one documented rule (§3.1
   already says this — pick *reject*, it's one line of code and zero ambiguity).
2. **Continuation suppression.** Inside `(` `[` `{`, and after a trailing binary operator
   or `,`, newlines and indentation are not significant. This is the rule `docs/01 - Redox.md`
   §4.2 gestures at ("Newlines terminate statements unless an expression is continued")
   and must be made precise — it is the single largest source of indentation-lexer bugs.
3. **Trivia capture.** Comments and blank-line runs become `Trivia` attached to the
   following token (§3.2). Never discarded.
4. **Adjacency flags.** Each token records whether whitespace preceded it, so the parser
   can distinguish `foo(x)` from `foo (x)` (§4.6) and `-x` from `a - x`.
5. **Multiline strings / heredocs.** `"""…"""` (§7.2) with common-indentation stripping.
   Emerald implemented `<<~IDENT` heredocs as a **raw-source preprocessing pass**
   (`crates/emerald-parser/src/heredoc.rs`, 447 lines) specifically because a dynamic
   terminator needs a regex backreference its lexer couldn't express — and it notes two
   wins worth copying: splicing the dedented body back as an ordinary string literal made
   `#{}` interpolation work inside heredocs *for free, with zero grammar changes*, and
   line numbers after the heredoc were preserved. Redox's `"""` terminator is static, so
   this can live in the lexer proper, but **preserve the line-number property** — it is
   load-bearing for §6.5's source maps.
6. **Interpolation.** `#{…}` must be lexed as a nested expression, not string-scanned.
   Emerald keeps this in a tiny 85-line `interpolate.rs`.

### 5.2 Parser — hand-written recursive descent + Pratt. Not LALRPOP.

Emerald chose LALRPOP over Chumsky with a documented table (§3.5). **Redox should choose
neither**, and the evidence is in Emerald's own record:

1. **LALR(1) is what blocked parenless calls for 89 plans.** The conflict at
   `grammar.lalrpop:622-626` is a pure artifact of one-token lookahead. Redox has parenless
   calls in its *core* spec (§6.1), not as a nicety.
2. **Indentation-sensitive grammars fit LR generators badly.** They need lexer/parser
   feedback, which is exactly what a generated parser makes awkward.
3. **Error recovery was retrofitted.** Emerald's plan 26 threads an
   `errors: &mut Vec<...>` parameter through the grammar to get multi-error recovery. A
   hand-written parser does this natively, and `docs/01 - Redox.md` §29 wants good diagnostics.
4. **Trivia preservation is the formatter's whole problem** (§3.2), and generated parsers
   are the reason it's hard.
5. **Redox's grammar is small.** It parses Redox-specific sugar and otherwise *preserves
   Rust expressions and types* (§3.2 of the spec). That is a small recursive-descent
   parser with a Pratt expression loop, not a 761-production inventory.

Follow §3.5's methodology anyway: **build the vertical slice twice** — hand-written vs
LALRPOP — on `docs/01 - Redox.md` §31's Phase 1 acceptance test, publish the comparison table in
`docs/COMPILER.md`, and record a revisit trigger. The recommendation above is a hypothesis
until that's done; Emerald's own history is the argument for not skipping the prototype.

**Operator precedence:** Emerald adopted Ruby's precedence table unchanged, reasoning there
was no motivation to diverge. Redox should adopt **Rust's** precedence table unchanged, for
a stronger reason: a Redox expression that reads like Rust and means something different is
the worst possible failure mode for a transpiler. Then add the two new operators with
explicitly stated precedence — `|>` (§15 of the spec notes it must not conflict with
bitwise `|`) and `??` (§13.2).

### 5.3 Emitter — not "codegen"

Replaces `emerald-codegen`'s 34,429 lines with, realistically, 2–4k lines. Responsibilities:
emit Rust source text; maintain the source map (§6.5); re-emit trivia; manage hygiene for
generated temporaries (§6.4). Deterministic output is a spec requirement (§3.4) and is
also what makes golden tests possible (§8.1).

**Do not run rustfmt on the output as part of the contract.** Emit canonical formatting
directly. Shelling out to rustfmt would make the compiler's output depend on an external
tool's version, break determinism, and destroy the source map. Offer it as
`redox build --rustfmt` for humans, applied *after* the map is written and marked as
invalidating the map.

### 5.4 Crate split

Emerald's 10-crate split is good and worth copying in shape. Its **one clear structural
defect is file size**: 34,429 lines in `emerald-codegen/src/lib.rs` and 22,589 in
`emerald-sema/src/lib.rs`, both single files. Compare `emerald-driver`, which is 2,658
lines across four well-named modules (`lib.rs`, `cache.rs`, `require_graph.rs`,
`parallel.rs`) and is by far the most readable crate in the repository.

```
redox/
├── crates/
│   ├── redox-lexer/      # off-side rule, trivia, adjacency, interpolation
│   ├── redox-ast/        # AST + spans + trivia; no logic
│   ├── redox-parser/     # recursive descent + Pratt; multi-error recovery
│   ├── redox-emit/       # Rust source emitter + source map
│   ├── redox-diag/       # miette integration, stable error codes
│   ├── redox-driver/     # orchestration, blake3 query cache, parallel, rustc remap
│   ├── redox-fmt/        # canonical .sdx formatter (trivia-preserving)
│   ├── redox-cli/        # build / check / fmt / watch
│   ├── redox-lsp/
│   └── redox-mcp/        # optional
├── examples/             # *.sdx — CI-checked source of truth (§8.3)
├── tests/
│   ├── golden/           # .sdx → expected .xrs
│   ├── ui/               # .sdx → expected stderr (diagnostics)
│   └── compile/          # generated .xrs must compile and run
├── fuzz/                 # thin libfuzzer wrappers (§8.5)
├── tree-sitter-redox/
├── editors/
├── docs/
│   ├── 01 - Redox.md                   # the specification
│   ├── 02 - Redox Update Blueprint.md  # this document
│   ├── 03 - Redox Surface Design.md    # the surface, with a lowering per construct
│   ├── 04 - Redox Plan of Plans.md     # index-only plan table (§3.1)
│   ├── 05 - NN - <Name>.md             # one per plan row; 05 - 01 is the exemplar
│   ├── COMPILER.md                     # proposed: toolchain decisions + revisit triggers (§3.5)
│   ├── GRAMMAR.md                      # proposed: graded inventory, both axes (§4.1)
│   ├── DECLINED.md                     # proposed: permanent declines, with reasons (§3.3)
│   └── STATUS.md                       # proposed: measured claims + gap table (§3.4)
```

**Add a CI check capping any single `.rs` file at ~1,500 lines.** It is the cheapest
possible defence against the one structural problem Emerald demonstrably has.

**Superseded in part, 2026-11-08.** The line cap shipped, but it is now the *backstop*
rather than the primary rule. `docs/06 - Redox Code Architecture.md` §1 requires **one
construct per file** — every `struct`, `enum`, `trait`, `type`, `fn`, `const` and `static`
in its own file named after it — enforced by
`crates/redox-cli/tests/one_construct_per_file.rs`. Under that rule a file rarely
approaches 1,500 lines, and when one does it signals that the single construct inside it is
doing too much. Doc 06 also fixes the module-root, re-export, visibility, error-handling and
test-placement conventions this section left implicit.

### 5.5 Dependencies

Emerald's choices, and the Redox verdict:

| Concern | Emerald | Redox |
|---|---|---|
| Lexing | `logos` | **none** — hand-written (§5.1) |
| Parsing | `lalrpop` + `lalrpop-util` | **none** — hand-written (§5.2) |
| Diagnostics | `miette` 7 (`fancy`) + `thiserror` 2 | **adopt both.** `docs/01 - Redox.md` §29's example output *is* miette's format. |
| Incrementality | hand-rolled `blake3` query cache, explicitly not `salsa` | **adopt** (§7.1) |
| Parallelism | `std::thread::scope` + `Mutex<VecDeque>`, explicitly not `rayon` | **adopt the reasoning**; Redox may not need it at all (§7.2) |
| Orchestration | `id_effect` 0.4, adopted "where it clarifies the architecture, not everywhere" | optional; the scoping clause is the useful part |
| Watching | — | `notify`, as §2 of the spec already allows |
| LSP | `lsp-server` + `lsp-types` | adopt |
| MCP | `rmcp` 3.2 + `schemars` + `tokio` | optional (§9.3) |

That is 3–5 production dependencies, consistent with §2's "Avoid parser generators and
unnecessary compiler frameworks" and §30's minimality constraint. Note Emerald's
`emerald-fuzz-smoke` went further — **zero external dependencies**, hand-writing a 30-line
SplitMix64 mutator rather than pulling `rand`/`arbitrary`, specifically so adding the crate
added exactly one `Cargo.lock` entry and no new third-party code to `cargo audit`'s
surface. Good instinct to copy where it's cheap.

---

## 6. The lowering discipline — Redox's central design problem

This section has no Emerald analogue, and it is where Redox will actually succeed or fail.
It deserves its own normative document. **Delivered as `docs/03 - Redox Surface Design.md`**,
whose §§5–15 give a lowering per construct; the separate `docs/LOWERING.md` proposed here is
superseded rather than still owed.

### 6.1 The problem

`docs/01 - Redox.md` §30 forbids a type inference engine, and §33 forbids changing Rust's type
or ownership semantics. But several specified constructs appear to *need* type knowledge:

- §9.1 lowers `[n * n for n in 1..10]` to `(1..=10).map(…).collect()` — but §9.2 lowers
  `[n for n in numbers when …]` to `numbers.into_iter().filter(…).collect()`. Which
  adapter? That depends on what `numbers` is.
- §10 says object comprehensions "should use `std::collections::HashMap` unless a type
  context specifies another map." **There is no type context available.**
- §13.1 `?.` → "equivalent `Option::and_then` and `Option::map` operations." Which one
  depends on whether the method returns an `Option`.
- §13.2 `??` → "`unwrap_or` or `unwrap_or_else`, as appropriate." Appropriate by what test?
- §14 chained comparison must evaluate intermediates once — requires knowing whether the
  value is `Copy`.

### 6.2 The rule

> **The no-inference lowering rule.** Every Redox construct lowers to Rust whose meaning is
> determined by *rustc's* inference, never by Redox's guess. Where a lowering would need a
> type to choose between two shapes, choose a third shape that needs none.

Two corollaries:

> **Every check rustc can make, rustc makes.** Redox performs no type check, no borrow
> check, no exhaustiveness check, no mutability check. If a user writes `counter += 1`
> against an immutable binding, Redox emits `let counter` and lets rustc say so — remapped
> to the `.sdx` line by §6.5. This is why Redox needs no `redox-sema` crate at all, and
> why it can be 8k lines where Emerald is 107k.
>
> **Ownership stays syntactic.** `docs/01 - Redox.md` §9.7 is already the model: the *user*
> writes `for x in items` / `&items` / `&mut items`, and the transpiler mechanically emits
> `into_iter` / `iter` / `iter_mut`. No inference, no silent clone (§25.1), user intent
> explicit at the site. **Apply that same discipline to every construct**, and treat any
> construct that can't be made to work this way as a candidate for §12.4's decline list.

### 6.3 Worked lowerings

These are the ones the spec currently under-specifies. Proposed as normative.

**Comprehensions — turbofish, never a `let` annotation.** §9.1's example output
(`let squares: Vec<_> = …`) only works in a `let` with no other annotation; it breaks the
moment the comprehension appears in argument position. Always emit a self-contained
expression:

```rust
(1..=10).map(|n| n * n).collect::<Vec<_>>()
```

And per §6.2, the iterator source comes from what the user wrote, not from a guess:
`for n in items` → `items.into_iter()`; `&items` → `items.iter()`; `&mut items` →
`items.iter_mut()`; a range literal → the range expression directly. Object comprehensions
emit `.collect::<std::collections::HashMap<_, _>>()`; "unless a type context specifies
another map" must be **deleted from §10** and replaced with an explicit opt-in spelling if
another map type is ever wanted.

**Optional chaining — delegate to Rust's `?` inside a closure.** This is the key trick, and
it resolves §13.1 with zero type knowledge. `name = user?.profile?.name` becomes:

```rust
let name = (|| Some(user.as_ref()?.profile.as_ref()?.name.clone()))();
```

rustc performs the entire `and_then` / `map` unification itself, via `?` in a
`-> Option<_>` closure. Nested options are not flattened (satisfying §13.1's requirement)
because `?` doesn't flatten. The `.clone()` on the tail is the one open question — it
contradicts §25.1's no-implicit-clone rule, so the final projection should probably be
left to the user (`user?.profile?.name.clone()`), with the lowering emitting only the
chain. **Decide and record this** (§13).

**Null-coalescing — `match`, not `unwrap_or`.** §13.2's "`unwrap_or` or `unwrap_or_else`,
as appropriate" cannot be decided without types, and `unwrap_or_else`'s closure arity
differs between `Option` (zero args) and `Result` (one). So: `??` is **`Option`-only** by
specification, and lowers to a `match`:

```rust
match a { Some(__rdx_v) => __rdx_v, None => b }
```

No method resolution to get wrong (a user type with its own inherent `unwrap_or_else`
can't hijack it), correct laziness for `b`, and rustc's error message if `a` isn't an
`Option` points at the `match`, which the source map resolves to the `??`.

**Chained comparison — bind by reference.** §14 requires single evaluation. Binding by
reference works for `Copy` and non-`Copy` alike, so no type knowledge is needed:

```rust
{ let __rdx_c0 = &age; 0 < *__rdx_c0 && *__rdx_c0 < 120 }
```

**Pipeline — and a defect in §15's example.** §15 shows:

```
result = numbers |> filter (n) -> n > 0 |> map (n) -> n * n |> sum
```

Under the stated semantics ("the preceding expression is passed as the first argument to
the following function") this lowers to `sum(map(filter(numbers, …), …))` — **and none of
`filter`, `map`, or `sum` exist as free functions in Rust.** The example cannot compile.
Fix one of three ways:

- **(a)** Keep first-argument semantics, and replace the example with one using real free
  functions. Iterator chaining then uses ordinary `.method()` chains, which Rust already
  does well. *Recommended — smallest, no new semantics, no ambiguity.*
- **(b)** Make `|>` a *receiver* pipeline: `x |> map(f)` → `x.map(f)`. More useful for
  iterators, but then `|>` is just `.` with different whitespace rules, which is hard to
  justify.
- **(c)** Drop `|>` entirely and add it to `docs/DECLINED.md`. Defensible: Rust's method
  chains already cover the motivating case.

**Hygiene.** Every generated binding uses the reserved prefix `__rdx_`, and
`redox-parser` **rejects any user identifier beginning with `__rdx_`** with a named
diagnostic. One lexer rule, and generated-temporary capture bugs become impossible.

### 6.4 What minimal static analysis Redox still needs

Not a type checker. A small resolver, for the cases where the *desugaring itself* needs
structure:

1. **Binding scope**, only to make comprehension/loop variables and generated temporaries
   correct.
2. **Receiver context** for `@` / `@@` — §16 says these transformations "are valid only in
   receiver contexts," which requires knowing whether the enclosing item is an `impl`
   method. Purely syntactic; no types.
3. **Module-scope vs function-scope**, because top-level `let` is illegal in Rust (§10.5).
4. **Reserved-prefix rejection** (§6.3).

That's a few hundred lines. Anything beyond it is drift toward `emerald-sema`, and should
be challenged in review.

### 6.5 Source maps — the largest single gap in the current spec

`docs/01 - Redox.md` §29 says: *"Pass Rust errors through wherever possible."* As written, that
means the user sees:

```
error[E0308]: mismatched types
  --> main.xrs:47:19
```

pointing at a **line of generated code they did not write**. For a transpiler this is the
difference between a usable tool and a toy, and it is currently unaddressed.

Emerald's analogue is instructive: plan 35 (`debug-info`) emits real DWARF line tables via
inkwell's `DebugInfoBuilder` (`compile_to_object_with_debug_info`) so a debugger maps
machine code back to `.em` lines — and it was *"proven with a scripted debugger session,"*
not by inspection. Redox needs the same capability one layer up, and the same standard of
proof.

**Design:**

1. **Line-preserving emission wherever feasible.** Make one `.sdx` line emit one `.xrs`
   line as the default discipline. This is achievable for most constructs and makes the
   map almost trivial. Emerald's heredoc pass makes the same choice ("line numbers after a
   heredoc are preserved") for the same reason.
2. **A real span map for the rest.** The emitter records `(gen_start, gen_end) ↔
   (src_start, src_end)` pairs. Serialize to a `main.xrs.map` sidecar.
3. **Remap rustc's diagnostics.** `redox check` runs `cargo check --message-format=json`,
   parses the diagnostic stream, rewrites every span through the map, and re-renders via
   miette against the original `.sdx`. Emerald already parses Cargo's NDJSON artifact
   stream in `crates/emerald-driver/build.rs` for a different purpose, so the technique is
   proven in-tree.
4. **Prove it like Emerald proved DWARF:** a test that compiles a `.sdx` with a *deliberate
   type error on a known line*, runs real rustc, and asserts the remapped diagnostic
   reports that exact `.sdx` line and column. Not an inspection — an executed test.

This single feature is worth more to Redox's usability than every item in Phase 5 of the
spec's roadmap.

### 6.6 Cargo integration — also unresolved

§1.1 flags it honestly (*"The compiler must account for Rust compiler input-file extension
handling"*) and then leaves it open. It must be closed before Phase 2, because it
determines what a Redox project even looks like. rustc accepts `.rs`; the options:

- **(a) `build.rs` + `OUT_DIR` + `include!`.** A user's crate has a `build.rs` calling the
  Redox library over `src/**/*.sdx`, emitting `.rs` into `OUT_DIR`, with thin `mod`
  wrappers using `include!`. Pure-Cargo, no new tooling, incremental rebuilds handled by
  Cargo, and `include!` preserves spans to the generated file so §6.5's map still works.
  *Recommended.*
- **(b) A parallel generated tree** (`src-gen/`) checked in or gitignored, with `.rs`
  extensions. Simple, inspectable; risks staleness.
- **(c) A `cargo redox` subcommand** wrapping the whole build. Most control, most work,
  and §1.3 says "no build system replacement."

Under (a), `.xrs` becomes the *canonical human-facing artifact* (`redox build` writes it
for inspection and review) while the build path writes `.rs` into `OUT_DIR`. That keeps
§37's chemistry pun and the `.xrs` convention without fighting rustc.

---

## 7. Driver: incrementality and parallelism

`emerald-driver` is the crate to copy most closely.

### 7.1 The query cache

`crates/emerald-driver/src/cache.rs` (742 lines) is a hand-rolled three-query memoization
layer. Its own doc comment names the design and the deliberate non-choice:

> the query-based architecture rustc's own `rustc_query_system` uses internally and the
> pattern the `salsa` crate generalizes … applied by hand rather than by adding that crate,
> since this plan's whole-file granularity doesn't need `salsa`'s fine-grained machinery.
> **A cache MISS always falls back to calling the real function verbatim — a cache hit is
> strictly an optimization, never a correctness dependency.**

Two details worth stealing outright:

- **`blake3::Hash` as the cache key with zero wrapper boilerplate** — already
  `Copy`/`Eq`/`Hash`/`Display`-as-hex.
- **The compiler's own binary is hashed into every key** (`compiler_fingerprint()` reads
  `current_exe()` and hashes it), *"so that rebuilding the compiler invalidates every
  previously-cached result instead of silently reusing a stale `.o` a different compiler
  produced."* This is exactly the bug a Redox watch loop would otherwise hit after
  `cargo build`, and it's three lines.

Also copy the `CacheReporter` trait: the cache reports `[cache] <query> <label> HIT|MISS`
and *has no opinion on stdout* — the CLI decides verbose vs silent. That's what makes
`--verbose-cache` a one-line feature and makes cache behaviour testable.

For Redox the queries are `parse(file)` and `emit(file)`. This is also what makes §28's
watcher correct rather than merely fast — "Avoid unnecessary recompilation" (§28) is a
cache-hit property, not an ad-hoc mtime check.

### 7.2 Parallelism — probably not yet

`crates/emerald-driver/src/parallel.rs` (356 lines) schedules per-file queries onto a
bounded `std::thread::scope` pool, with Kahn's-algorithm levelling from the require graph
and a hard barrier between levels. The scheduler is deliberately *"a `Mutex<VecDeque<PathBuf>>`
drained by a fixed number of workers"* — no `rayon`.

Redox has an easier problem (transpilation is per-file and dependency-free; there is no
cross-file symbol resolution to order) and a much smaller payoff (text emission, not LLVM
O3). **Defer parallelism**, and when it's needed, note that Emerald's design collapses to a
plain parallel map with no levelling at all. Take the one orthogonal idea now though:
`EMERALD_TRACE_COMPILE=1` dumps per-file `(file, thread, start, end)` records as a
*concurrency proof seam* — the general habit of building a measurement seam alongside a
performance feature is worth keeping.

---

## 8. Testing

Emerald's ladder, adapted. This is the second-highest-value transfer after §1.1.

### 8.1 Golden tests — `.sdx` → expected `.xrs`

`docs/01 - Redox.md` §3.4 already requires deterministic, readable output; that makes exact-text
golden tests possible, and they are the cheapest regression net a transpiler can have.
`tests/golden/<name>.sdx` + `<name>.xrs`, with a `--bless` flag to regenerate.

### 8.2 Compile tests — the generated Rust must actually compile

§32 already asks for this ("Test both: Generated Rust output; Successful Rust
compilation"). Make it mechanical: every golden `.xrs` is compiled by real rustc in CI.
Golden text proves *stability*; compilation proves *correctness*. Neither substitutes.

### 8.3 Examples as the CI-checked source of truth — adopt wholesale

This is Emerald's best practice. `README.md`:

> `examples/` is the actual, CI-checked source of truth for what works today — every file
> there is compiled, run, and asserted against its real output on every push

`crates/emerald-cli/tests/examples.rs` runs the real built binary via
`env!("CARGO_BIN_EXE_emerald")` over all 135 files and asserts **exact stdout**. And
`examples/README.md` holds a per-file table of what each one covers *and* the gaps and bugs
found while writing them. `spec/SPEC.md`'s reading order ends by pointing a new contributor
at that file, with the reason stated plainly:

> for what's actually proven to work today, as opposed to what's merely documented — **the
> two are not the same set**, and that file is where the difference is tracked honestly.

For Redox each example is compiled to `.xrs`, **compiled by rustc, run, and its stdout
asserted**. And per §4.5: **examples are files, never string literals in Rust tests.**

### 8.4 UI tests for diagnostics

§29 specifies diagnostic *formatting* with an example rendering. Formatting specified
without a test is formatting that rots. `tests/ui/<name>.sdx` + `<name>.stderr`, in rustc's
own style. This is also the only way to keep §6.5's source-map remapping honest.

### 8.5 Fuzzing — copy the architecture, not just the idea

Emerald's `emerald-fuzz-smoke` (1,861 lines) has an unusually good structure, documented in
its `Cargo.toml`:

- **All harness logic lives in a normal, stable-toolchain crate**; `fuzz/fuzz_targets/*.rs`
  are three-line `libfuzzer_sys::fuzz_target!` wrappers. So the generators and mutator are
  exercised by ordinary `#[test]`s under the **default** test gate on stable, instead of
  only under a nightly `cargo fuzz` run nobody remembers to do.
- **Zero external dependencies** (§5.5).
- Targets: `lex_source`, `parse_program`, `sema_check`, `generated_program`. Redox's are
  `lex_source`, `parse_program`, `emit_program`, plus a **generated-program** target — and
  Redox gets a much stronger oracle than Emerald had: *the generated Rust must parse.*
  Pipe fuzz output through rustc's parser; a panic or a syntax error in generated Rust is
  always a Redox bug.

Property tests worth having: lexer indentation round-trips; the formatter is idempotent
(`fmt(fmt(x)) == fmt(x)`); transpilation is idempotent-stable; and
`parse(fmt(x)) ≡ parse(x)` modulo trivia.

### 8.6 Zero-cost must be measured

Emerald's benchmark discipline — *"6 programs × 10 runs × 6 languages, methodology
disclosed in full"* — with results that openly include losing to Crystal on four of six, is
the standard to match. The Redox analogue is **not** language-vs-language; it's:

1. **Transpile throughput** — lines/second, and watch-loop latency.
2. **Zero-cost verification** — for each benchmark program, a hand-written Rust version and
   a Redox-generated version, compiled identically, asserted to be within noise. Redox's
   entire value proposition is "no cost versus writing Rust by hand." That is a *claim*
   until it's a *measurement*. If a comprehension lowering is slower than the hand-written
   loop, that's a bug with a number attached.

---

## 9. Tooling surface

Emerald shipped a complete toolchain — `new`, `build`, `run`, `test`, `property`,
`benchmark`, `doc`, `lint`, `format`, `repl`, `update` (`crates/emerald-cli/src/main.rs`),
plus LSP, MCP, tree-sitter, and VSCode/Neovim/Helix/Zed configs. **Redox should ship much
less**, and `docs/01 - Redox.md` §27's "Do not add unnecessary commands" is right: Cargo already
owns build, test, doc, dependency management, and publishing. Redox's CLI stays
`build` / `check` / `fmt` / `watch` / `--version` / `--help`.

But three pieces of Emerald's surface are worth more to Redox than its own spec currently
credits:

### 9.1 LSP — add to the roadmap

`emerald-lsp` is **545 lines** on top of the driver and delivers diagnostics, goto-
definition, completion, and semantic tokens. For an indentation-sensitive language with a
brand-new syntax, live diagnostics are close to a precondition for anyone trying it. 545
lines is a remarkable ratio, and Redox's version is simpler — it has no symbol table to
build (§6.2), so it's diagnostics + semantic tokens + a formatter hook.

Also heed plan 90's warning (§4.6): check for `(`-assuming textual heuristics before
parenless calls land.

### 9.2 Formatter — needed earlier than it looks, and must be trivia-preserving

`docs/01 - Redox.md` doesn't mention a formatter. Three independent reasons it belongs in Phase
2–3, not "later":

1. **It's the migration tool** (§4.5). Emerald's 113-missed-fixture cutover is the
   cautionary tale.
2. **Indentation-significant syntax makes a formatter unusually load-bearing** — mixed
   tabs/spaces and inconsistent continuation indentation are *syntax* errors here, not
   style.
3. **It forces the canonical-spelling decision early** (§4.6), while the cost of deciding
   is still low.

**And it must not repeat `emerald-fmt`'s architecture.** Emerald chose a pure AST
pretty-printer and lost every non-doc comment (§3.2). Redox's formatter must be
trivia-preserving — which it gets for free if §5.1's lexer and §3.2's AST do their jobs,
since the transpiler needs trivia anyway. One investment, two consumers.

### 9.3 MCP — cheap, and this repo is already agent-heavy

`emerald-mcp` is 392 lines exposing the driver over MCP. Given this repo's `.mcp.json`,
`.maestro/`, and `.claude/` setup, an agent-facing `redox check`/`redox build` tool is
plausibly worth 400 lines. Low priority; low cost; note it.

### 9.4 tree-sitter + editors

Emerald's plan 24 shipped a real `grammar.js`, generated parser, highlight queries, and
corpus tests. For Redox this is table stakes for anyone to *read* `.sdx` — and tree-sitter
corpus tests are an independent check on the grammar's shape, which is a side benefit worth
having. Note the duplication cost honestly: an indentation-sensitive tree-sitter grammar is
genuinely fiddly, and it's a second grammar to keep in sync with §4.1's inventory.

---

## 10. Specific corrections to `docs/01 - Redox.md`

Each item is a proposed, reviewable edit.

### 10.1 §4.1 — comments are not removed

**Current:** "Comments are removed during compilation."
**Change to:** comments are trivia attached to AST nodes and are re-emitted into the
`.xrs`. `#` → `//`; `###…###` → `/* … */`; the doc-comment form → `///` (and `//!` for
inner). Rationale: §3.2 above.

### 10.2 §23 — attributes are missing entirely, and `#` collides

The spec covers macros but never Rust **attributes**. Without them a Redox user cannot
write `#[derive(Debug, Clone)]`, `#[cfg(test)]`, `#[tokio::main]`, `#[serde(rename)]`, or
`#[allow(...)]` — which makes §26's interoperability requirement unmeetable in practice.

There's also a direct lexical collision: `#` opens a comment (§4.1), `#{` opens
interpolation (§7.1), and Rust's attributes are `#[`.

**Proposed resolution:** a lexer rule — `#[` begins an attribute (passed through verbatim
to the emitter), `#{` begins interpolation inside a string, `#` otherwise begins a comment.
One token of lookahead, no ambiguity. Then add an `§23.2 Attributes` section with the
passthrough rule, including inner attributes (`#![…]`) and their placement constraints.
If the lookahead is judged too subtle, the alternative is to respell Redox's comment
character — a larger change, decided now rather than later (§4.5).

### 10.3 §6.3 — `!` for zero-arg calls collides with macros

§6.3 makes `!` a zero-argument call suffix (`user.save!` → `user.save()`) while §6.3's own
next paragraph preserves Rust macro syntax (`println! "Hello"`, `vec! [1, 2, 3]`). So
`ident!` is ambiguous: zero-arg call, or macro invocation?

Emerald's parallel case is instructive — it supports *bare* zero-arg dotted calls
(`p.distance_from_origin`, no sigil at all), which is cleaner, but it can only do that
because its type checker knows whether the name is a field or a method. **Redox cannot**:
`user.name` (field) and `user.save` (method) are syntactically identical and must lower
differently. So Redox genuinely needs the sigil.

**Proposed resolution:** keep `!`, and disambiguate by lookahead — `ident!` followed by
`(`, `[`, `{`, a string literal, or an expression *on the same line* is a macro invocation;
`ident!` at the end of an expression is a zero-argument call. Write this rule into §6.3
explicitly. Then fix the two spec examples that contradict it or the path syntax:
`Vec.new!` (§6.3) and `Vec::new!` (§4.4) disagree — see §10.4.

### 10.4 §§4.4 / 6.3 / 13.3 / 22 — path separator is inconsistent

The spec uses, in different places:

- `String::from "Alice"` (§4.4) — Rust `::`
- `Vec.new!` (§6.3) vs `Vec::new!` (§4.4) — **direct contradiction**
- `fs.read_to_string path?` (§13.3) — dot for a module path
- `import std.collections.HashMap` (§22) — dots for a use path

Three different conventions for one concept. A transpiler cannot infer which dots are
`::` and which are field access (§10.3's problem again, one level up).

**Proposed resolution:** `::` everywhere in expressions, matching Rust exactly and making
`.` unambiguously field-or-method access. For `import`, either adopt `::` too (maximal
consistency) or keep dots *only* inside the `import` statement, where no expression grammar
applies and the rule is trivially unambiguous. Decide, then sweep every example in the
spec.

### 10.5 §4.3 — top-level bindings must be `const`/`static`, and need annotations

§4.3 says uppercase names generate `const`. Two problems:

1. **`let` is illegal at Rust module scope.** So *every* top-level `name = value` must
   become `const` or `static`, regardless of case.
2. **Rust `const`/`static` require an explicit type** — `const PI: f64 = 3.14159;`. The
   type cannot be elided, so the spec's `PI = 3.14159` cannot be lowered at all.

**Proposed resolution:** module-scope bindings require an explicit type annotation
(`PI: f64 = 3.14159`), lower to `const` by default, and add a spelling for `static` and
`static mut`. Say explicitly that the uppercase convention is a lint, not semantics.

### 10.6 §5.5 — decline default arguments

§5.5 proposes default parameters and hedges ("can be postponed"). Recommend **declining
permanently** and recording it in `docs/DECLINED.md`. The generated "convenience wrapper"
creates a second function that is not the one the user named, which breaks trait impls
(you cannot add a wrapper to a trait method), confuses rustdoc and go-to-definition,
multiplies monomorphizations, and makes diagnostics point at a function the user never
wrote. Emerald could afford defaults because it owned the whole pipeline; Redox would be
fabricating API surface. Point users at `Option<T>` parameters or a builder.

### 10.7 §20 — decline or defer `with` expressions, explicitly

§20 already hedges ("optional," "If implementation requires complex semantic analysis,
postpone it") and its own constraint ("must not introduce hidden copies, implicit mutable
references, or changes to borrowing semantics") is very hard to meet without types —
resolving a bare `host = …` inside `with config` to `config.host = …` requires knowing
`config`'s fields. Move to `docs/DECLINED.md` as `UNDECIDED` with a named resolution venue
(§3.3), rather than leaving it as a half-specified feature.

### 10.8 §15 — the pipeline example cannot compile

See §6.3 above. Pick (a), (b), or (c); (a) recommended.

### 10.9 §3.3 — narrow the `print` mapping

§3.3's table maps `print` → `println!`, and the spec's examples use it throughout. That
makes `print` a hardcoded keyword — **exactly** Emerald's `puts` situation
(`grammar.lalrpop:622-626`), which papered over a grammar conflict for 89 plans (§4.6).
Two independent problems: it's a built-in name in a language that claims no standard
library (§1.2), and it hides the ambiguity instead of resolving it.

**Proposed resolution:** solve general parenless calls properly (§4.6 option B), then
`print x` needs no special case — it's an ordinary parenless call to whatever `print`
resolves to. Keep *one* mapping, `print` → `println!`, documented as the single sanctioned
sugar, or drop it and have examples use `println!` directly. Do not accumulate a second.

### 10.10 §12 / §21 — state the splat-vs-range rule

§21 makes `1...10` an exclusive range; §12 makes `...numbers` a splat and `[first, ...rest]`
a rest pattern. §12 says only that "the parser must distinguish" them.

**Proposed resolution:** write the rule down — `...` is a splat/rest **only in prefix
position** inside a collection literal, argument list, or pattern; it is a range operator
**only in infix position**. Then `[1...10]` is unambiguously a range and `[0, ...xs, 4]`
unambiguously a splat. Also specify the half-open forms (`1...` → `1..`, `...10` in
range position vs splat position) or decline them.

### 10.11 §5.4 — define "receiver context" for `=>`

§5.4 says `->` is an ordinary closure and `=>` is "a receiver-capturing closure when used
in a receiver context," which is undefined. §16 uses the same undefined phrase for `@`/`@@`.
Define it once — *inside the body of a method in an `impl` or `class` block* — and
cross-reference from both sections. Also specify how `move` closures are spelled, since
they're unavoidable with threads and `async`.

### 10.12 §9.1 vs §9.2 — unify the comprehension lowering

The two examples disagree on iterator adapter and on collect form. Replace both with §6.3's
rule. Also fix §9.1's range: `[n * n for n in 1..10]` is shown lowering to `(1..=10)`,
which is consistent with §21's inclusive `..` — but §32's test example repeats the same
pattern, so keep the two in sync when editing.

### 10.13 §29 — specify the remapping, and a stable error-code catalog

§29's four categories are right, but "pass Rust errors through wherever possible" must
become the §6.5 remapping design. Also add stable diagnostic codes
(`RDX0001`…) in `docs/DIAGNOSTICS.md` — Emerald's inception §14.4 sets this bar
(*"Diagnostics are a first-class compiler feature, not an afterthought"*) with a worked
`error[E0007]` rendering, and codes are what make §8.4's UI tests assert something durable.

---

## 11. Revised roadmap

`docs/01 - Redox.md` §31's seven phases are reasonable but omit source maps, Cargo integration,
comments, attributes, the formatter, and the LSP — and defer diagnostics to Phase 6, which
is too late to be designed well.

| Phase | Deliverable | Acceptance |
|---|---|---|
| **0 — Decide** | `docs/GRAMMAR.md` graded inventory, both axes (§4.1). The lowering rules of §6.3 — **done, as `docs/03 - Redox Surface Design.md`**. `docs/COMPILER.md` with the parser bake-off (§5.2) and revisit triggers. `docs/DECLINED.md`. Answer §13's open questions — **done, doc 03 §§1–22**. | Every §10 correction resolved in the spec. No `UNDECIDED` without a named venue. |
| **1 — Vertical slice** | `redox-lexer` (off-side, trivia, adjacency), `redox-ast`, `redox-parser`, `redox-emit`, `redox-cli build`. Golden + compile + run tests from day one. | §31 Phase 1's `add`/`main` program transpiles, **compiles under rustc, runs, prints 30**. Comments survive into `.xrs`. |
| **2 — Diagnostics and maps** | `redox-diag` (miette, `RDX` codes), source map, `cargo check --message-format=json` remapping, `redox check`. UI tests. | A deliberate type error in `.sdx` reports **the correct `.sdx` line and column** via real rustc (§6.5's executed test). |
| **3 — Cargo integration** | §6.6's decision, implemented. `build.rs` integration crate or equivalent. | A `cargo build` on a mixed `.sdx` + `.rs` project works, incrementally, with correct diagnostics. |
| **4 — Core syntax** | Implicit calls (§4.6 option B), zero-arg calls (§10.3's rule), interpolation, if/unless/while/until/for, postfix forms, `switch`. | §31 Phase 2's list, every item with an example + golden + compile + run test. |
| **5 — Rust surface** | Structs, enums, traits, impls, generics, lifetimes, imports/exports, macros, **attributes (§10.2)**, `where` clauses, async/await, inline Rust. | Every `UNREACHABLE` row in §4.1's table is either reachable or on the decline list. A real project using a crates.io dependency builds. |
| **6 — Collections and sugar** | Comprehensions (§6.3), destructuring, splat (§10.10), ranges, slices, `@`/`@@`, `?.`/`??` (§6.3), `do`, chained comparison, pipeline (§10.8). | Zero-cost benchmark (§8.6): generated vs hand-written within noise. |
| **7 — Tooling** | `redox-fmt` (trivia-preserving), `redox watch` (on §7.1's cache), `redox-lsp`, tree-sitter + editors. | Formatter idempotent and comment-preserving; watch loop sub-100ms on a single-file edit. |
| **8 — Hardening** | Fuzz targets (§8.5) incl. generated-Rust-must-parse, property tests, `docs/STATUS.md`. | Fuzzers run clean; status doc's numbers reproducible by the stated command. |
| **9 — Conformance** | Compiler's own source in Redox (§4.8). | Compiles itself **without using the inline-Rust escape hatch**; generated `.xrs` committed. |

Phases 0–3 are the ones that decide whether Redox is pleasant or painful. The original
roadmap's Phase 1 is this one's Phase 1, but phases 2 and 3 are new and come *before* the
language surface grows — deliberately, because retrofitting source maps onto an emitter
that wasn't built for them is the kind of rework Emerald's own history keeps showing
(diagnostic spans, plan 22: *"`TypeExpr` carries no span at all, so every diagnostic
hardcoded `(0, 0)`"* — found 21 plans after the diagnostics plan, then fixed with a
scoped workaround rather than the full refactor, which was *"judged too large/risky"*).

---

## 12. Compatibility rules, extended

`docs/01 - Redox.md` §33's ten mandatory rules are good. Proposed additions, each traceable to
something above:

11. **No lowering may depend on a type Redox cannot see** (§6.2).
12. **Every diagnostic a user sees names a `.sdx` location** (§6.5).
13. **Comments and doc comments survive into the generated Rust** (§3.2).
14. **Every generated identifier carries the `__rdx_` prefix; user identifiers may not**
    (§6.3).
15. **No Redox plan implements a library function** (§1.1).
16. **Every Rust construct is reachable** — via first-class syntax or the documented escape
    hatch, and the escape hatch's use for a common construct is a bug report against the
    grammar (§4.1).
17. **Generated code is as fast as hand-written Rust, and this is measured** (§8.6).

### 12.4 Initial decline list

Proposed contents of `docs/DECLINED.md`, each with a reason:

| Feature | Status | Reason |
|---|---|---|
| Default arguments (§5.5) | **declined** | Fabricates API surface; breaks trait impls, rustdoc, go-to-definition (§10.6) |
| `with` expressions (§20) | **UNDECIDED** → venue: Phase 6 review | Needs field knowledge; can't meet its own no-hidden-borrows constraint (§10.7) |
| Standalone `value?` existential | declined (already in §13) | Ambiguous with Rust's `?` |
| `extends` / inheritance | declined (already in §17.7) | Rust has no inheritance; traits + composition are the answer |
| Implicit cloning | declined (already in §25.1) | Non-negotiable |
| `|>` pipeline | **UNDECIDED** → venue: §10.8 | Example doesn't compile; Rust method chains may already cover it |
| A Redox type checker | **permanently declined** | §6.2; this is the project's load-bearing constraint |
| A Redox standard library / prelude | **permanently declined** | §1.1 |

---

## 13. Open questions needing a decision

Answers change the work; defaults are proposed, but these are the owner's calls. Run the
genuinely contested ones per §3.6.

1. **Parenless-call strategy** — adjacency lexing (§4.6 option B, recommended) vs
   statement-initial-only? Option A breaks §6.4's own example.
2. **Comment and attribute lexing** — `#[` / `#{` / `#` lookahead (§10.2, recommended), or
   respell Redox's comment character now?
3. **Path separator** — `::` everywhere (recommended), and what does `import` use (§10.4)?
4. **Cargo integration** — `build.rs` + `OUT_DIR` + `include!` (recommended), parallel
   generated tree, or a `cargo redox` subcommand (§6.6)?
5. **Is `.xrs` the build input or a human-facing artifact?** The recommendation in §6.6
   makes it the latter; confirm, because §1.1 and §37 read as the former.
6. **`@@` for `Self::`** — keep, given MoonScript's and Ruby's `@@` means a class variable
   and Emerald deleted it as a footgun (§4.4)?
7. **`?.` tail projection** — does the lowering emit `.clone()` (convenient, violates
   §25.1) or require the user to write it (consistent, more verbose)? (§6.3)
8. **`??` scope** — `Option`-only via `match` (recommended), or extend to `Result`
   (needs types; probably impossible) (§6.3)?
9. **Pipeline** — keep with a corrected example, respell as a receiver pipeline, or decline
   (§10.8)?
10. **Doc-comment spelling** — what maps to `///` and `//!`? Emerald used `##` and it's the
    one comment form its parser retains; Redox needs both inner and outer forms.
11. **Self-hosting** — accept §4.8's reframing from architecture goal to conformance test?
12. **Parser technology** — accept §5.2's hand-written recommendation, or run the bake-off
    first (§3.5 says run it)?

---

## 14. Summary

The three things Emerald is most worth copying:

1. **Its library story, inverted.** 112 of 203 plans went into a standard library. Redox's
   charter already gives it crates.io for free; §1.1's rule — *no Redox plan ever
   implements a library function* — is the most valuable single line in this document.
2. **Its process.** Graded grammar inventory (§4.1), plan-of-plans with honest stale-label
   corrections (§3.1), examples as CI-checked truth (§8.3), prototype-before-committing
   with written revisit triggers (§3.5), measured claims and recorded tooling traps (§3.4).
   All free.
3. **Its driver.** `cache.rs` and `parallel.rs` are ~1,100 lines of directly reusable
   design, including the compiler-binary-in-the-cache-key trick (§7.1).

The three things Emerald most warns against:

1. **Owning semantics.** `emerald-sema` is 22,589 lines because Emerald must answer "what
   does this mean?". Redox must never answer that question (§6.2).
2. **Single-file growth.** 34,429 lines in one `lib.rs`. A CI line cap costs nothing (§5.4).
3. **Discarding trivia, and deferring span infrastructure.** Comments lost at the lexer
   (§3.2); `TypeExpr` spans missing until 21 plans after the diagnostics plan, then patched
   rather than fixed. Both are retrofits Redox can avoid by building trivia and source maps
   into Phase 1–2 (§§5.1, 6.5).

And the single largest gap in `docs/01 - Redox.md` today: **§29's "pass Rust errors through"
has no mechanism.** Without source maps (§6.5), a Redox user debugs generated code. Emerald
solved the analogous problem with DWARF line tables and *proved it with a scripted debugger
session*. Redox needs the same capability and the same standard of proof, and it needs it
before the language surface grows.
