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
careful that this is passing a function, not currying one. It does not make a bare Redox name a
value.

But the hazard is real without plan 56's help, and already present: `Effect::retry probe_rate`
works **today**, because a bare Redox name in argument position is already the function item.
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

Measured 2026-11-10, against `redox` at `194fdf7`.

**A parameterless binding is a function and its use site calls it.** `examples/module.sdx` has
`print (sizes ())`; `examples/strings.sdx` has `print (block ())`; `examples/constants.sdx` has
`print MAX` for the `const` beside them. Three spellings for the same idea in three adjacent
files.

**A bare name emits the function item**, which rustc rejects today with *"use parentheses to
call this function"* — plan 20 §10.6's finding, re-measured and unchanged.

**The factory pattern depends on the bare name, today.** Verified running:
`rate = Effect::retry probe_rate (Schedule::recurs 4) |> flat_map parse_rate` makes four attempts
against a flaky upstream and succeeds, where a single shared description could not. `suspend` is
what defers the body; the binding is what rebuilds it.

**`f ().method ()` misparses.** `drain ().len ()` emits `drain(().len())` — a lone `()` absorbed
as the argument. Plan 57 removes the method-call side of this; the *call* side is this row's,
because `drain` stops needing `()` at all.

**Corpus scale.** **39** `name ()` call sites on parameterless Redox bindings across the new
corpus's 905 code lines — one every twenty-three lines — plus `sizes ()`, `block ()`,
`inner::hello ()` and `inner::deeper::deep ()` in the examples. The `()` is not a rare
ceremony.

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

### Leaf 2 — `leaf-const-emission`

A shared binding emits `const`, which is doc 03 §13.4's default. `static` keeps its explicit
keyword — §13.4 states no rule selecting it by type, and inventing one here would make the
keyword's meaning conditional. Reuses plan 16's emission rather than adding a second path.

### Leaf 3 — `leaf-call-elision`

A function binding's use site emits the call. The name alone is the whole surface; the
parentheses are gone from both sides.

### Leaf 4 — `leaf-explicit-keyword-still-works`

`const` and `static` with the keyword keep working and keep meaning exactly what they mean. A
keyword on a non-const expression is a diagnostic, which is rustc's error today and becomes a
Redox one.

### Leaf 5 — `leaf-corpus-and-docs`

`examples/module.sdx`, `strings.sdx`, `constants.sdx` and every `name ()` site. Plan 20 §10.6 is
amended — its finding was correct and its rule is superseded. Doc 03 §13.4 and doc 18 §7 updated;
`docs/GRAMMAR.md` regraded.

---

## 5. Acceptance criteria

1. A parameterless binding is used by name, with no parentheses, at every use site including
   module-qualified ones.
2. A literal binding emits `const`; a working binding emits `fn` plus an elided call. Both
   asserted in goldens, so the classification is visible.
3. `Effect::retry probe_rate (…)` makes four attempts against the flaky-upstream fixture — a
   **run** assertion, per §3 case 3.
4. `name ()` with explicit parentheses is a diagnostic naming the bare form, because every
   program written before this row uses it.
5. Explicit `const` and `static` are unchanged; a keyword on a non-const body is a Redox
   diagnostic.
6. All 29 examples green, no rustc output, stdout unchanged.
7. `construct_coverage` green.
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

Doc 03 §13.3's decline of transpile-time evaluation is Emerald's lesson stated in Redox's
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

*Not yet executed.*
