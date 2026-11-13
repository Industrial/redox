# Plan 59 — Value Bindings

Plan ID: `value-bindings`
Depends on: 55 (`expression-orientation`), 56 (`currying`)
Milestone: 9 — *surface coherence*
Status: `planned`

---

## 1. What this plan is

A top-level binding with no parameters is a **value**. You write `block`, not `block ()`.

Today `name : T` plus `name = e` compiles to `fn name() -> T`, so every use site calls it. Plan
20 §10.6 recorded the rule and was right about it:

> Doc 18 §7 replaced doc 03 §5.2's `!` zero-argument suffix with `f ()`, so the call is spelled
> `settle ()`: juxtaposition needs something to juxtapose, and a lone `()` is the empty argument
> list. The example was wrong, not the emitter.

The rule is coherent. What it leaves is **two kinds of top-level value with different use-site
syntax** — `const MAX : usize = 1024` is `MAX`, and `block : &'static str` is `block ()` — and
the difference is invisible at the use site and has nothing to do with what the author meant.

### 1.1 The split is an artefact, not a design

Doc 03 §13.4 corrected doc 01 §4.3 twice: Rust forbids `let` at module scope, and `const` and
`static` need explicit types. The conclusion was that *every* top-level binding is `const` or
`static`.

Plan 16 implemented that for bindings carrying the keyword. A binding **without** the keyword
becomes a function *when a signature line precedes it* — and plan 16 leaf 3 gave the
signature-less case its own diagnostic, `RDX0106`, so `PI = 3.14159` at module scope is refused
rather than silently anything. The split this row closes is therefore between a keyword-carrying
binding and a signature-carrying one:

```
const MAX : usize = 1024        ->  const MAX: usize = 1024;      -- use: MAX
block : &'static str            ->  fn block() -> &'static str    -- use: block ()
block = """…"""
```

Both are values in the source. One needs parentheses. Nothing in docs 01, 03, 18 or plan 16
argues for the difference.

### 1.2 Currying makes it worse, and that is why this row is here now

Plan 56 §1.5 makes a **Rust path** usable as a function value — `map String::from` — and is
careful that this is passing a function, not currying one. It does not make a bare Voltaic name a
value.

But the hazard is real without plan 56's help, and already present: `Effect::retry probe_rate`
works **today**, because a bare Voltaic name in argument position is already the function item.
That is what every `Effect::retry` factory in the new corpus relies on. So `block` without
parentheses is not a parse error — it is a function item passed where a value was meant, and
after plan 56 the surrounding code is far more likely to accept it quietly.

This row therefore closes a hazard the milestone *sharpens* rather than one it creates, and it
must land after plan 56 so the two readings of a bare name are settled together.

### 1.3 Const-evaluable or a function, decided syntactically

The chosen rule is: a binding whose right-hand side is a **literal or a literal aggregate** is
shared; everything else is a function whose call is elided at the use site.

```
block : &'static str = """…"""      -- literal       -> const, shared
sizes : usize = .len a + .len b     -- has a call    -> fn, elided call
```

This matters because the obvious alternative — "memoise anything const-evaluable" — requires
**const evaluation**, and doc 03 §13.3 declines that permanently and in these words:

> Zig- and Emerald-style transpile-time evaluation is **declined**: it would require shipping a
> literal-only interpreter, a scope expansion of the same order as the type checker doc 02 §12.4
> declines permanently, and Emerald's own plan 61 needed a step-limit flag for it.

So the rule is a **syntactic** test — does the right-hand side contain an application? — and not
an evaluation. No interpreter, no step limit, no scope expansion. The cost is that some
genuinely-constant expressions become functions, which is a performance decision rustc makes
anyway through inlining.

### 1.4 Memoisation is refused, and a real program is why

The alternative on the table was Haskell's: a zero-argument binding is evaluated once and shared,
like a CAF, via `LazyLock` or `OnceLock`.

That breaks id_effect. An `Effect` is a **one-shot description**: running it consumes it, and
anything that re-runs takes a factory. A program written against the current surface has

```
probe_rate : Effect<String, ProbeError, HNil>
probe_rate = suspend rate_attempt

measured = Effect::retry probe_rate (Schedule::recurs 4) |> …
```

`Effect::retry` calls `probe_rate` **once per attempt** and needs a fresh description each time.
Memoising the binding would hand it the same consumed effect five times — compiling, running,
and retrying nothing.

So sharing is confined to literals, where it is indistinguishable from `const`, and every binding
whose body does work is recomputed per use. That is the current behaviour, with the parentheses
removed, which is the smallest change that fixes §1.1 without breaking §1.4.

---

## 2. Verified preconditions

Measured 2026-11-10, against `voltaic` at `194fdf7`.

**A parameterless binding is a function and its use site calls it.** `examples/module.sdx` has
`print (sizes ())`; `examples/strings.sdx` has `print (block ())`; `examples/constants.sdx` has
`print MAX` for the `const` beside them. Three spellings for the same idea in three adjacent
files.

**A bare name emits the function item**, which rustc rejects today with *"use parentheses to
call this function"* — plan 20 §10.6's finding, re-measured and unchanged.

That sentence is true of raw rustc and false of Voltaic, which §11.6 is the finding for: `check.rs`
passes `--error-format=short` and `RustcMessage::parse` keeps only positioned diagnostics, so
`voltaic check` reports `E0277 … doesn't implement Display` and nothing else. **A Voltaic user never
sees the parentheses line**, which matters because "rustc already tells them" is part of this
row's motivation.

**The factory pattern depends on the bare name, today.** Verified running:
`rate = Effect::retry probe_rate (Schedule::recurs 4) |> flat_map parse_rate` makes four attempts
against a flaky upstream and succeeds, where a single shared description could not. `suspend` is
what defers the body; the binding is what rebuilds it.

**`f ().method ()` misparses.** `drain ().len ()` emits `drain(().len())` — a lone `()` absorbed
as the argument. Plan 57 removes the method-call side of this; the *call* side is this row's,
because `drain` stops needing `()` at all.

**Corpus scale.** At least **13** `name ()` call sites on parameterless Voltaic bindings in
`examples/` alone — eight in `examples/strings.sdx` (`nested`, `pattern_text`, `quoted`,
`initial`, `tag`, `magic`, `ffi_name`, `block`) and five in `examples/module.sdx`. The `()` is
not a rare ceremony.

Corrected from *"**39** `name ()` call sites … across the new corpus's 905 code lines — one every
twenty-three lines — plus `sizes ()`, `block ()`, `inner::hello ()` and `inner::deeper::deep ()`
in the examples"*: §11.4 counts the examples at three times the four sites that sentence named,
and §11.5 found the corpus the 39 came from **is not in this repository** — `id_effect`,
`Effect::retry` and `Schedule::recurs` appear in documents only.

---

## 3. Concrete proof this plan targets

**`print block` compiles**, where `print (block ())` was required — and `print MAX` beside it is
unchanged, so the two kinds of top-level value finally read the same.

Five cases:

1. **A literal binding becomes a `const`.** Asserted in a golden: `block` emits `const`, not
   `fn`.
2. **A binding whose body does work stays a function.** `sizes` emits `fn sizes()` and the use
   site emits `sizes()`. Also asserted in a golden, because this is the half a reader will not
   expect.
3. **The factory still works.** `Effect::retry probe_rate (…)` makes four attempts. This is the
   §1.4 regression test and it must be a *run* assertion — a memoised version compiles.
4. **A module-qualified value.** `inner::hello`, no parentheses, which is where the elision has
   to survive a path.
5. **A genuine `() -> T` function is still callable.** A parameterless *function* the author
   wants to call explicitly — the distinction between a value and a nullary function — is
   recorded in §6 as unresolved and asserted to behave as case 2.

---

## 4. Leaves

### Leaf 1 — `leaf-literal-classification`

The syntactic test from §1.3: does the right-hand side contain an application? Literals, literal
aggregates, paths to other constants — shared. Anything else — a function. One pass over the
declaration's expression, no evaluation.

The classification has a precedent and **one property the precedent lacks**, which §11.3 is the
finding for. `names_a_constructor` classifies a record head by case alone and drives two
lowerings — *"decidable from the source text alone, which is doc 03 §2's admission test 1 and the
reason this needs no name resolution"* — and `ascribed_list` and `expr_list` are two more. All
three classify **locally**. This one does not: the declaration's classification determines every
use site's spelling, and that cross-site agreement is a new property rather than an inherited
one. It is where the row's risk actually lives.

Placement is settled by plan 56 §11.3 and is cited rather than re-derived: the **check** rides
`parse()`, the declared chokepoint; the **rewrite** rides the emitter's entry. Not `parse()`
alone, because `voltaic_fmt::format` consumes the same `Program`.

### Leaf 2 — `leaf-const-emission`

A shared binding emits `const`, which is doc 03 §13.4's default. `static` keeps its explicit
keyword — §13.4 states no rule selecting it by type, and inventing one here would make the
keyword's meaning conditional. Reuses plan 16's emission rather than adding a second path.

**Sharing additionally requires the name to be upper case**, per §11.8: `const block` earns
`non_upper_case_globals` and the examples corpus requires rustc to emit nothing, so a
lower-case literal binding stays a `fn` with its call elided. The use site is identical either
way, which is why this narrows the leaf without touching criterion 1.

### Leaf 3 — `leaf-call-elision`

A function binding's use site emits the call. The name alone is the whole surface; the
parentheses are gone from both sides.

**Bounded to the file being compiled**, per §11.1: bare names, and paths into an **inline**
`module` with a body. A bodiless `module` has `items: None` and `emit_module` writes `;`, and
`Expr::Path` is verbatim text with no structured name to resolve, so the referent is not in this
`Program` and Voltaic cannot know to elide the call. Outside that set the use site keeps today's
parenthesised spelling, which is the fallback plan 56 §11.8 had and this row's first draft
removed.

### Leaf 4 — `leaf-explicit-keyword-still-works`

`const` and `static` with the keyword keep working and keep meaning exactly what they mean. A
keyword on a non-const expression stays **rustc's error**, which is the status quo.

Amended from *"a keyword on a non-const expression is a diagnostic, which is rustc's error today
and becomes a Voltaic one"*, which §11.2 found rejects a program that compiles today — see
criterion 5, where the measurement is.

### Leaf 5 — `leaf-corpus-and-docs`

`examples/module.sdx`, `strings.sdx`, `constants.sdx` and every `name ()` site. Plan 20 §10.6 is
amended — its finding was correct and its rule is superseded. Doc 03 §13.4 and doc 18 §7 updated;
`docs/GRAMMAR.md` regraded.

**At least 13 call sites, not four** — §11.4's count, listed in §2. And two statements of plan
52's fallout live in files this row already touches, so they are this leaf's: `const_decl.rs`'s
doc comment and `examples/constants.sdx` both still say the type goes *"after the same `::`"*
(§11.7).

---

## 5. Acceptance criteria

1. A parameterless binding is used by name, with no parentheses, at every use site **in the file
   being compiled** — bare names, and paths into an **inline** `module` with a body.

   Amended from *"at every use site including module-qualified ones"*, which §11.1 found leaves a
   cross-file binding with no spelling at all: `Expr::Path` is carried verbatim as a `String`, a
   bodiless `module` has `items: None` and `emit_module` writes `;`, so the referent is not in
   this `Program` and Voltaic cannot know to elide the call. Measured there: `print inner::probe`
   emits `println!("{}", inner::probe)` → `E0277: fn() -> u32 {probe} doesn't implement Display`.
   Plan 56 §11.8 withdrew the same claim for the same reason; criterion 4 narrows to the same set
   so that the parenthesised spelling survives outside it.
2. A literal binding **whose name is upper case** emits `const`; a working binding, and a
   literal binding with a lower-case name, emit `fn` plus an elided call. All three asserted in
   goldens, so the classification is visible.

   Amended from *"A literal binding emits `const`"*, which §11.8 measured as turning
   `examples/strings.sdx` red: its binding is `block`, and `const block` warns.
3. `Effect::retry probe_rate (…)` makes four attempts against the flaky-upstream fixture — a
   **run** assertion, per §3 case 3.

   §11.5 found **the fixture does not exist and cannot live in `examples/`**. `id_effect`,
   `Effect::retry` and `Schedule::recurs` appear in documents only — zero hits under `crates/`,
   `examples/` or any `Cargo.toml`, which has no `[workspace.dependencies]` section at all. And
   `voltaic check` shells out to bare rustc with `--edition 2024 --error-format=short
   --emit=metadata` and **no `--extern`**, so an external-crate fixture needs the
   `examples/cargo-project/` shape — `Cargo.toml`, `build.rs`, `include!`, driven by
   `compile_dir` — a project that has no parameterless binding and no `module` today. On top of
   that infrastructure the criterion wants a **nondeterministic upstream** that must still
   satisfy a deterministic `*.stdout` gate.

   §11.5 stopped at *"state that or drop it"*, which is two options and not a decision.
   **Taken in §11.9: neither — replaced.** The criterion now reads: a parameterless binding
   passed bare to a higher-order function declared in the same file is invoked **once per call**,
   witnessed by a `run` fixture whose body prints, so a memoised binding prints once and a
   rebuilt one prints twice. Same property, deterministic stdout, no external crate. The
   `id_effect` program stays in §1.4 as the motivation it always was.
4. `name ()` with explicit parentheses is a diagnostic naming the bare form **where the callee is
   a bare name, or a path into an inline `module`, declared parameterless in this file** —
   because every program written before this row uses it. Everywhere else `()` keeps being erased
   exactly as today.

   Amended from *"`name ()` with explicit parentheses is a diagnostic naming the bare form"*,
   which §11.1 found fires across the whole language: `()` is erased for every call on one line.

   ```rust
   // emit/expr_list.rs
   if matches!(args, [Expr::Unit { .. }]) { return String::new(); }
   ```

   `HashMap::new ()`, `String::new ()`, `.len ()`, `.is_ok ()` and `sizes ()` all travel through
   it, so the diagnostic must fire for the last and stay silent for the rest. The narrowing is
   criterion 1's set, which is also the only set this row can decide.

   **One residual is owed**, and §11.1 names it rather than settling it: inside that set a lambda
   parameter, an equation binder or a `where` local sharing a name with a parameterless binding
   is indistinguishable from the binding itself at the call site — the undecidability plan 56
   §11.1 withdrew its leaf 3 over. Whether leaf 3 keeps the diagnostic silent in the presence of
   a local shadow, or the diagnostic goes the way of plan 56's leaf 3, is not decided here.
5. Explicit `const` and `static` are unchanged. A keyword on a body §1.3's syntactic test calls
   non-const is **left to rustc**, which is the status quo: the genuinely non-const case is
   `E0015: cannot call non-const function`, remapped onto the `.sdx` at the right column.

   Amended from *"a keyword on a non-const body is a Voltaic diagnostic"*, which §11.2 found
   rejects a program that compiles today:

   ```text
   square : const u32 -> u32
   const SQ : u32 = square 7     ->  const SQ: u32 = square(7);   exit 0, clean
   ```

   `square 7` contains an application, so §1.3's syntactic test calls it non-const and the
   original criterion refuses it; `const N : usize = size_of::<u32>()` is the same shape. That is
   a check rustc makes, and makes well, so the original breaks AGENTS.md's standing rule — *every
   check rustc can make, rustc makes* — and costs working programs. Leaf 4 is amended to match.
6. Every example green, no rustc output, stdout unchanged.

   **No count, deliberately.** This criterion said *"All 29 examples green"*, and the corpus has been 29, then 30, then 31 during this milestone alone — `examples/local.sdx` arrived with plan 55 leaf 1. A number here is a criterion that a future example breaks. The invariant is what matters and it is already enforced: `examples.rs` runs every file and excepts exactly the ones its `NOT_PARSEABLE` constant names, today `["lexical.sdx"]`, the error-path example. `docs/STATUS.md` is generated and is where the count lives.
7. `construct_coverage` green — near-free as the row stands, per §11.7: coverage keys off
   `docs/GRAMMAR.md` rows, `const fn` and `const`/`static` are already witnessed by
   `examples/constants.sdx`, and nothing grades `f ()`. It stops being free if leaf 5 adds a row.
8. Plan 20 §10.6 and doc 18 §7 are both amended, each citing this row.

---

## 6. Out of scope, and what it discloses

**A value and a nullary function are now indistinguishable at the use site.** Both are written
`name`. If someone genuinely wants `() -> T` as a type — a thunk to pass around — they write it
as a lambda (`\-> e`, plan 64) or a one-parameter function taking unit. The collapse is
deliberate and it is the thing that makes the surface uniform, but it means the language no
longer has a way to *say* "nullary function" in a binding. Recorded as the real cost.

**Sharing is confined to literals, so Haskell's CAF semantics are not available.** A binding that
does expensive work is recomputed at every use. That is today's behaviour and the row does not
regress it, but a reader coming from Haskell will expect sharing and not get it. The honest
alternative — memoise and break factories — is §1.4.

**No const evaluation.** §1.3, and doc 03 §13.3 declines it permanently. A binding like
`2 * 1024` contains no application and so is shared; `double 1024` contains one and is not. That
boundary is syntactic and slightly arbitrary, and it is the price of not shipping an interpreter.

**The `f ()` ambiguity is only half closed here.** This row removes the call-site parentheses;
plan 57 removes the method-call ones. Neither alone makes `drain ().len ()` unwritable — together
they do, and §2 records which half is which.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout unchanged;
the id_effect factory fixture runs and makes four attempts; goldens re-blessed and reviewed;
`construct_coverage` green.

---

## 8. Emerald reuse

Doc 03 §13.3's decline of transpile-time evaluation is Emerald's lesson stated in Voltaic's
documents: *"Emerald's own plan 61 needed a step-limit flag for it."* This row is the first to be
designed **around** that decline rather than merely respecting it — §1.3's syntactic test exists
because the evaluating version is the thing Emerald paid for.

That is the transfer: not a technique, but a declined technique whose decline shapes a design.

---

## 9. Decision log

1. **Values, not `name ()`.** §1.1. The split is an artefact of which row shipped which half.
2. **Literal-or-function, not const-evaluation.** §1.3. The evaluating rule needs an interpreter
   doc 03 §13.3 declines permanently; the syntactic rule needs one pass.
3. **No memoisation.** §1.4. Verified against a real program: memoising breaks every
   one-shot-description idiom, and id_effect's `Effect` is the one this repository has a corpus
   for.
4. **Elide the call rather than make the binding a `LazyLock`.** Same reasoning as 3, and it
   keeps the generated Rust the Rust a person would write — no hidden global, no initialisation
   order.
5. **`name ()` becomes a diagnostic, not silently the same thing.** Every pre-existing program
   writes it, and the error message is the migration guide — the same choice plan 52 made for the
   spaced `::`.
6. **Land after plan 56.** §1.2: a bare name is already a function item today, and plan 56 makes
   the positions that accept one much commoner. The hazard is sharpened rather than created, and
   the two readings of a bare name should be settled in one milestone.

---

## 10. Result

**Done 2026-11-10**, against `edf3bb3`. Leaves 1, 2, 3 and 5 shipped; the gate is green and
**no golden moved for any pre-existing example**, which is plan 52's definition of correctness
for a surface row and the strongest thing this row can say.

### 10.1 The bare and parenthesised spellings emit the same Rust, deliberately

Criterion 4 asks for a diagnostic on `name ()`. The emitter instead makes both spellings read
the binding **once**, and that ordering was not in the plan: routing a callee through
`binding_text` and then applying it writes `block()()`, so the Call arm had to recognise the
parenthesised form before `emit_application` ever saw it.

Keeping both working is what made the migration free. Every one of the corpus's pre-existing
`name ()` sites emitted byte-identical Rust before and after, so leaf 5's migration —
`examples/module.sdx`'s four sites and `examples/strings.sdx`'s one — moved no golden either.
**The diagnostic is therefore outstanding, not shipped**, and it is now a deprecation of a
spelling that works rather than a repair of one that miscompiles. That is a better thing to
own and a smaller one.

### 10.2 Two grammar gaps, both invisible, both found by one example

`examples/bindings.sdx` is the first corpus file to write a **nullary lambda**, and the
tree-sitter grammar had `repeat1` on a lambda's parameters — so `\->` was an `ERROR` node for
every editor client from the moment it became reachable in the surface. Nothing had reported
it because nothing in the corpus wrote one.

The second is this row's own doing. Sharing requires an upper-case name (§11.8), an upper-case
name lexes as a `type_identifier`, and both `signature` and `equation` bound their name field to
`$.identifier` — so `LIMIT : usize` and `LIMIT = 1024` were `ERROR` nodes while the compiler
accepted them happily. Both rules now admit either spelling, which is the authority rule
`AGENTS.md` records: the compiler's table decides and the grammar describes.

Three ERROR nodes, found by `every_example_parses_with_no_error` and located with
`tree-sitter parse` — which is the habit that gotcha asks for, working.

### 10.3 What the generated documents say, and whether it is the expected direction

`docs/CONFORMANCE.md`: `voltaic-emit` gains 6 files and 2 blocked, so its blocked share falls
**73% → 71%** and the macro count rises 941 → 944. Four of six new files carry no blocker,
which is the direction a row adding declaration-shaped code should move.

`docs/STATUS.md`: `voltaic-emit` +381 source lines, examples 34 → 35, goldens 33 → 34.

### 10.4 Leaf 4 cost nothing, which §11.2 predicted

`const` and `static` with the keyword are untouched, and a keyword on a non-const body is still
rustc's `E0015`. No code was written for this leaf because §11.2 had already established that
the right behaviour was the existing one — which is what an orientation is for.

---

## 11. Corrections from orientation

Written after reading the code and probing it. Nine of eleven preconditions hold
exactly as §2 states them. **Two acceptance criteria cannot both be satisfied,
and a third rejects a program that compiles today.**

### 11.1 Criteria 1 and 4 leave a cross-file binding with no spelling

Criterion 1 wants the bare name *"at every use site including module-qualified
ones"*. Criterion 4 makes `name ()` a diagnostic. For a **bodiless** `module`
those two together leave nothing that works:

- `a::b::c` is `Expr::Path { text: String }` — carried **verbatim**, built in
  `path_or_ident.rs` and emitted as `text.clone()`. There is no structured name
  to resolve.
- A bodiless `module` has `items: None` and `emit_module` writes `;`, so the
  referent is not in this `Program` at all. Measured: `print inner::probe` emits
  `println!("{}", inner::probe)` → `E0277: fn() -> u32 {probe} doesn't implement
  Display`.
- So Voltaic cannot know to elide the call — and criterion 4 has just forbidden
  the author from writing it.

**Plan 56 hit the same wall and withdrew the claim**; §11.8 of that row records
it. The difference is that plan 56 has a safe fallback — *emit a call exactly as
today* — and this row does not, because criterion 4 removes the other spelling.

Criterion 1 must narrow to **bare names and inline-module paths in the file
being compiled**, and criterion 4 to the same set.

**And criterion 4 has a second problem: `()` is erased for every call in the
language, on one line.**

```rust
// emit/expr_list.rs
if matches!(args, [Expr::Unit { .. }]) { return String::new(); }
```

`HashMap::new ()`, `String::new ()`, `.len ()`, `.is_ok ()` and `sizes ()` all
travel through it. A diagnostic there must fire for `sizes ()` and stay silent
for the rest — including for a lambda parameter, an equation binder and a
`where` local that happens to share a name — which is exactly the undecidability
plan 56 §11.1 withdrew its leaf 3 over.

### 11.2 Criterion 5 rejects a program that compiles today

*"A keyword on a non-const body is a Voltaic diagnostic."* Measured:

```text
square : const u32 -> u32
const SQ : u32 = square 7     ->  const SQ: u32 = square(7);   exit 0, clean
```

It contains an application, so §1.3's syntactic test calls it non-const and
criterion 5 refuses it. `const N : usize = size_of::<u32>()` is the same shape.
That is a check rustc makes — and makes well: the genuinely non-const case is
`E0015: cannot call non-const function`, remapped onto the `.sdx` at the right
column. Criterion 5 as written breaks AGENTS.md's rule and costs working
programs; the honest version is *leave it to rustc*, which is the status quo.

### 11.3 The classification has a precedent, and one property the precedent lacks

`names_a_constructor` classifies a record head by case alone and drives two
lowerings, with the rationale this row wants verbatim — *"Decidable from the
source text alone, which is doc 03 §2's admission test 1 and the reason this
needs no name resolution."* `ascribed_list` and `expr_list` are two more.

**But all three classify *locally*.** None has to agree with a decision made at
another site. This row's does: the declaration's classification determines every
use site's spelling. That is a new property, not an inherited one, and it is
where the row's risk actually lives.

Placement is already settled by plan 56 §11.3 and should be cited rather than
re-derived: the check rides `parse()`, the rewrite rides the emitter's entry, and
not `parse()` alone because `voltaic_fmt::format` consumes the same `Program`.

### 11.4 The migration is three times the stated size

§2 names four sites — `sizes ()`, `block ()`, `inner::hello ()`,
`inner::deeper::deep ()`. The real count in `examples/` alone is **at least 13**:
eight in `examples/strings.sdx` (`nested`, `pattern_text`, `quoted`, `initial`,
`tag`, `magic`, `ffi_name`, `block`) and five in `examples/module.sdx`.

### 11.5 Criterion 3's fixture does not exist, and cannot live in `examples/`

`id_effect` appears in **documents only** — zero hits under `crates/`,
`examples/`, or any `Cargo.toml`, which has no `[workspace.dependencies]`
section at all. `Effect::retry` and `Schedule::recurs` appear nowhere outside
plan docs. The corpus §2 measures — *"1,501 lines of new Voltaic … seven driving
`id_effect 0.13.0`"* — **is not in this repository**.

And `examples/` cannot hold it: `voltaic check` shells out to bare rustc with
`--edition 2024 --error-format=short --emit=metadata` and **no `--extern`**. An
external-crate fixture needs the `examples/cargo-project/` shape — `Cargo.toml`,
`build.rs`, `include!`, driven by `compile_dir` — and that project has no
parameterless binding and no `module` today. Criterion 3 is new infrastructure
plus a nondeterministic upstream that must still produce deterministic stdout
for the `*.stdout` gate. State that or drop it.

#### Decided 2026-11-10: criterion 3 is dropped, and criterion 4 silences on a shadow

**Criterion 3 goes.** §11.5 offered *"state that or drop it"*, and the deciding
factor is not the build-out cost — it is what the criterion would install. A
**nondeterministic upstream behind a byte-exact `*.stdout` gate is a flake
generator**, and this repository has already paid heavily for intermittent
failures it could not explain: AGENTS.md records a `watch.rs` test that fails at
exactly 20 seconds under load, and a tmpfs exhaustion that plan 23 §10.8, doc 30
§8.7 and plan 43 §10.3 each wrote down as *unexplained* before the mechanism was
found. Adding a deliberate source of flakiness to a gate is the wrong trade at
any price.

Two supports. The corpus the criterion needs **is not in this repository** —
§11.5 measured `id_effect` at zero hits outside `docs/` — so the criterion is
not merely expensive but currently unbuildable, and plan 64 §11.3's witness was
decided the same way on the same evidence. And §1.4's argument against
memoisation stands without a running external-crate fixture: it is an argument
about what a parameterless binding *means*, which a golden can assert.

**Criterion 4 keeps its diagnostic and silences it when an enclosing binder
shares the name**, which is §11.1's option (a). The asymmetry decides it: a
diagnostic that fails to fire costs an author one `()` they did not have to
remove, while one that fires wrongly **rejects a legal program**. That is the
same test that withdrew plan 56's leaf 3 — a false-positive generator is worse
than a gap — and applying it consistently is what makes the two rows coherent.

### 11.6 Plan 20 §10.6's quote is true of rustc and false of Voltaic

§2 says rustc *"rejects today with 'use parentheses to call this function' —
plan 20 §10.6's finding, re-measured and unchanged"*. Raw rustc does say it.
**A Voltaic user never sees that line**: `check.rs` passes
`--error-format=short` and `RustcMessage::parse` keeps only positioned
diagnostics, so `voltaic check` reports `E0277 … doesn't implement Display` and
nothing else. That matters, because "rustc already tells them" is part of this
row's motivation.

### 11.8 Leaf 2 emits a warning on the example it cites

Measured against the working tree after plans 55–67, by building the shape and running rustc on
the result:

```text
block : &'static str      ->  const block: &'static str = "literal";
block = "literal"
                              warning: constant `block` should have an upper case name
                              note: `#[warn(non_upper_case_globals)]` on by default
```

`examples/strings.sdx` is the file leaf 5 names and §2 cites, and its binding is `block` — lower
case. The examples corpus requires rustc to emit **nothing**, so leaf 2 as written turns that
example red on the row meant to improve it.

| Way out | Cost |
|---|---|
| synthesise `#[allow(non_upper_case_globals)]` | a line of noise per shared binding in a document doc 02 §10.1 makes human-facing — and the emitter would be *writing* an attribute where plan 53's only precedent *consumes* one |
| share only when the name is already upper case | the classification gains a second test, and an author writing `block = "x"` gets a `fn` where they might expect a `const`, with nothing at the use site to tell them |
| upper-case the emitted identifier | breaks leaf 3's inline-module paths, breaks the source map's name correspondence, and makes the `.xrs` say something the `.sdx` did not |

**The second is taken.** Case-driven classification is not new here: it is how doc 18 §5.3 tells
a type variable from a concrete type, how `primitive_type.rs` corrects that rule with a closed
list, and how `names_a_constructor` picks between two lowerings — which is leaf 1's own cited
precedent. And nothing a reader can see is lost, because the use site is `block` either way.
What changes is only which Rust item stands behind it.

The residual is recorded rather than hidden: a lower-case literal binding is recomputed per use
where an upper-case one is shared, and the surface does not say so. That is the same class of
invisible difference §1.3 already accepts — *"some genuinely-constant expressions become
functions, which is a performance decision rustc makes anyway through inlining."*

### 11.9 Criterion 3 is replaced, which is the decision §11.5 owed

The property criterion 3 protects is that a parameterless binding is **not** memoised: a factory
needs a fresh description per call. That property does not need `id_effect`, and §11.5's finding
is that `id_effect` cannot be had here — no `--extern` on `voltaic check`, no
`[workspace.dependencies]`, and a *nondeterministic* upstream that would still have to satisfy a
deterministic `*.stdout` gate.

What tests the same property with no dependency at all:

```
probe : u32
probe =
  print "built"
  7

twice : impl Fn() -> u32 -> u32
twice f = f () + f ()

main : ()
main =
  print (twice (\-> probe))
  ()
```

Memoised, that prints `built` once; rebuilt, twice. It also exercises criterion 4's narrowing in
the same fixture, because `f ()` is a **parameter** rather than a binding declared parameterless
in this file, so its parentheses must survive.

The thunk is explicit — `\-> probe` rather than bare `probe` — and §11.10 is why that is not a
detail.

### 11.10 Leaf 3 contradicts §1.4, and the thunk spelling already exists

**The contradiction, stated before it was built.** §1.4 refuses memoisation specifically to keep
`Effect::retry probe_rate` working, and §3 case 3 asserts it still makes four attempts. Leaf 3
appends the call to a bare parameterless name. Apply leaf 3 to §1.4's own program and it emits
`Effect::retry(probe_rate(), …)` — the factory receives a *consumed effect*, which is the exact
failure §1.4 exists to prevent. §6 records the value-versus-nullary-function question as
*"unresolved and asserted to behave as case 2"*, and case 2 is the call. So the row as published
protects a program in §1.4 and breaks it in leaf 3.

This is not decidable from syntax. `print block` wants the value and `retry probe_rate` wants the
function item, and both are a bare parameterless name in argument position.

**Resolved by spelling the thunk, which costs nothing because the spelling already parses.**
Measured against the working tree:

```text
twice (\-> probe ())   ->   twice(|| probe())
```

`\->` is the nullary lambda. It is in the surface today, so this needs nothing from plan 64 —
which matters, because plan 64 depends on *this* row and a resolution that needed it would be
circular.

So: a bare parameterless binding is its **value**, at every use site, which is criterion 1 and the
headline. A **factory** is `\-> name`, which says what it is. After this row that reads
`Effect::retry (\-> probe_rate) (Schedule::recurs 4)`, one token longer and no longer relying on
a bare name silently meaning something other than its value.

§6's residual is therefore **closed rather than carried**: the distinction between a value and a
nullary function is now spelled, not inferred. What remains open is only §11.1's narrower case —
a local binder shadowing a parameterless binding — which is about *which* `name` is meant rather
than about what a bare name means.

### 11.7 Smaller corrections

- §1.3's `block : &'static str = """…"""` and `sizes : usize = .len a + .len b`
  are **one-line forms the language does not have** for a signature-carrying
  binding; only `const` and `static` take one line. The real spelling is two,
  as `examples/strings.sdx` and `examples/module.sdx` write it. Minor, and it is
  the exact confusion this row is about.
- §6's *"they write it as a lambda (`\->`, plan 64)"* mis-cites: plan 64 is
  macro invocation, and lambdas already exist. A zero-parameter `\-> e` is plan
  64's *macro argument* spelling, which is not the same thing.
- **Verified exactly**, both quoted measurements: `drain ().len ()` emits
  `drain(().len())`, and a bare name in argument position is already the
  function item (`consume probe` → `E0308: expected u32, found fn item`).
- §2 cites commit `194fdf7`, which is 24 commits behind and **predates the type
  colon entirely**, so every `:` in a §2 code block is retrospective rather than
  measured.
- Criterion 7 is near-free: `construct_coverage` keys off `docs/GRAMMAR.md`
  rows, `const fn` and `const`/`static` are already witnessed by
  `examples/constants.sdx`, and nothing grades `f ()`. It stops being free if
  leaf 5 adds a row.
- `const_decl.rs`'s doc comment and `examples/constants.sdx` both still say the
  type goes *"after the same `::`"* — plan 52 fallout in files this row touches.
