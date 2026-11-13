# Declined

What Voltaic will not have, and why. Doc 03 §22 is the source; this restates it for a reader who
has not read a 2,000-line surface document, and
`crates/voltaic-cli/tests/declined.rs` asserts every §22 construct appears here so the two
cannot drift.

**A decline is not a gap.** A gap is a Rust construct a Voltaic program cannot express —
`docs/GRAMMAR.md` grades those `UNREACHABLE` and `tests/unreachable/` holds them failing. A
decline is a construct Voltaic has decided **not to offer**, usually one no other language's
version would survive doc 03 §2's admission test. Conflating the two makes both useless, which
is why they are separate documents.

Each entry is permanent unless a venue is named.

---

## Declined because it would imply a dependency

Doc 03 §2's second admission test: a construct's lowering may name nothing outside `core`,
`alloc` and `std`. A construct that needs a crate makes every Voltaic program depend on that
crate's author.

| Construct | From | Reason |
|---|---|---|
| Regex literals, extensible sigils | Ruby, Elixir | would imply an undeclared crate (doc 03 §11.3) |
| Property-test and benchmark blocks | Emerald | the same — a property block is `proptest`, a benchmark block is `criterion` (doc 03 §15) |
| Concurrent-await construct | — | every joining form is runtime-crate-provided, and doc 03 §14.3 ships `async` without an executor for that reason |

## Declined because it needs type information

Doc 03 §2's first admission test: the lowering must be determined by the **source text alone**.
A construct that needs to know a type needs a type checker, and doc 02 §12.4 calls the absence
of one the project's load-bearing constraint.

| Construct | From | Reason |
|---|---|---|
| UFCS | Nim | requires type knowledge to choose a shape (doc 03 §2) |
| A Voltaic type checker | — | the project's load-bearing constraint (doc 02 §12.4) |

> **One row has left this table, and it is the only decline in the document ever
> lifted.** *General monadic comprehension* (Scala) was declined because it
> *"must choose `flat_map` versus `and_then`"*, with the sole reopening venue
> here: **"revisit only if a type-free discrimination is found."**
>
> Plan 86 found one, and it was not new machinery. Plan 60 had already made `<-`
> monadic bind **read by the enclosing block modifier** — `try` reads it as `?`,
> `async` as `.await` — so a discrimination existed and the set it ranged over
> was merely closed. `do .and_then` and `do .flat_map` open it, and doc 03 §2
> clause 1 is satisfied in that clause's own words: *"the user's own syntax
> selects between them — never an inference."*
>
> Voltaic chooses nothing. It transcribes the method the author named, and
> fabricates nothing else either — the final row of the block is emitted as
> written, because choosing between `Ok`, `Some` and `iter::once` would be the
> inference the clause forbids.
>
> `examples/bind.sdx` is the witness, `docs/GRAMMAR.md` grades it, and the
> *Reopenable* table below is down to one entry.

> **Lifted 2026-11-11 by plan 90: custom operator declaration.** The row read
> *"no Rust lowering target exists (doc 03 §14.4)"*, and that **conflates two
> different things**. Rust has no custom-operator *mechanism*, which is true. It
> does not follow that there is no lowering *target*, and the surface already
> proves it: `|>`, `>>>`, `<<<`, `$`, `??` and `is` have no Rust counterpart at
> all, and every one of them lowers to an ordinary expression form. A declared
> `<+>` lowering to `merge(a, b)` has exactly the target `a |> f` has.
>
> What ships is **fixity plus alias**, not a definition: `infixl 6 <+> = merge`
> gives a spelling and a binding power to a function that already exists and
> emits no item. The Haskell form — where `(<+>)` *is* the name — stays out of
> reach, and for a reason this section's heading does cover: no Rust function can
> be named `<+>`, so the emitter would have to invent one, which is the
> fabricated API doc 03 §8.7 refuses for default arguments.
>
> `examples/operator.sdx` is the witness, `docs/GRAMMAR.md` grades it, and plan
> 90 §1.1 has the correction in full. This is the third row in this document
> whose *reason* was replaced rather than whose verdict was reversed on new
> evidence — plan 47 §1.4 restated two that cited a deleted section, and the
> turbofish row records a decline *"on a compiled measurement"*.

> **Added 2026-11-11 by plan 87: view patterns.** `case xs of (len -> 0) -> …`
> was leaf 3 of a shipped row, and it is refused rather than deferred — plan 87
> §10.6 has the measurement and this is where a reader will look for it.
>
> Rust's `match` **cannot call a function in pattern position**, so the only
> lowering is an `if let` / `else if let` chain — and **an `if let` chain is not
> exhaustiveness-checked, where the `match` it replaces is.** A `case` holding
> one view pattern would lose rustc's *"non-exhaustive patterns"* for *all* of
> its arms, not just the viewed one. That makes the compiler check **less** in
> exchange for a convenience, which `AGENTS.md`'s *"every check rustc can make,
> rustc makes"* forbids and which is the only place in the register that trade
> has been proposed.
>
> §1.3's proposed resolution — a mandatory trailing `_` arm, counted
> syntactically — is admissible and still a worse language, for the same reason.
>
> Two narrowings were considered and both are already expressible. A view
> applied to the **scrutinee** rather than per-arm keeps the `match`:
> `case (len xs) of 0 -> … ; n -> …` — and that is an ordinary `case` on an
> application, which Voltaic has had since plan 03. A view that returns a
> **bool** is a guard, which doc 18 §2.3 has. So there is nothing left to build
> that is not either a loss of checking or a spelling of something that works.
>
> What plan 87 did ship is pattern **synonyms**, which lower by textual
> substitution and so leave the `match` intact — `docs/GRAMMAR.md` grades them
> and `examples/synonym.sdx` is the witness.

## Declined because Rust has no target for it

A lowering needs something to lower *to*. These have no Rust form, so a Voltaic spelling would be
a feature of the compiler rather than of the language.

| Construct | From | Reason |
|---|---|---|
| Symbol literals and `%i[...]` | Ruby | Rust has no symbol type (doc 03 §10.4) |
| Two-binder `and` patterns | C# | no Rust pattern form binds from both sides (doc 03 §6.2) |
| Transpile-time evaluation (`comptime`) | Zig, Emerald | requires an interpreter in the compiler (doc 03 §13.3) |

## Declined because it duplicates something Voltaic already has

| Construct | From | Reason |
|---|---|---|
| Cascades | Dart, Smalltalk | method-chaining sugar with no Rust lowering of its own (doc 03 §7.5) |
| Scope functions (`apply`, `tap`, `doto`) | Kotlin, Ruby, Clojure | the same (doc 03 §7.5) |

> **Both reasons were restated by plan 47 §1.4.** Doc 03 §22 gave them as *"duplicates
> `with`"*, and doc 18 §7 **deleted** doc 03 §7.5's `with` blocks — reusing the keyword for doc
> 30 §3.3's callback bind. So the stated reason cited a construct that no longer existed.
>
> The verdict is unchanged, because it was never about the keyword: both forms are
> method-chaining sugar, and neither has a Rust lowering of its own.

## Declined because it would fabricate names or API

| Construct | From | Reason |
|---|---|---|
| `RecordWildCards` | Haskell | introduces names absent from the source (doc 03 §9.2) |
| A **functional** optic setter — `set`, `over` returning a new value | Haskell's `lens` | **inserts a clone**, which doc 03 §2 clause 3 forbids in as many words — *"never inserts a clone (doc 01 §25.1)"*. Plan 88 measured the alternatives and there are only three: clone the value and mutate the clone (clause 3); clone only when `T: Clone` and otherwise move, which is a choice made from a *type* (clause 1); or `..base` struct-update syntax, which reaches one level and is already doc 03 §9.1's spread. So this is an **admission failure rather than a scope decision**, and no later row can take it without relaxing a clause. What plan 88 shipped instead is the composed *getter*, `(.a.b.c)`, which inserts nothing, and — **since 2026-11-11** — the *mutating* `over`, which is the same ergonomics with none of the cost: `over (.addr.zip.code) doubled target` says the path once and lowers to an assignment through the place, so nothing is moved and nothing is cloned. The distinction this row turns on is therefore **functional versus mutating**, not *optic versus no optic*: `set`/`over`-returning-a-new-value stays refused on clause 3, and the in-place form is `docs/GRAMMAR.md`'s *Functional update through a path*. Plan 54's `:=` is still the general assignment, reaching a name, a field, an index and a deref target |
| Default arguments | C#, Ruby, Python | fabricates API surface (doc 03 §8.7) |

> **Second reason added 2026-11-11, and it is independent of the first.** Default arguments were
> re-proposed with the condition *"only if that works with currying"*. **They do not**, and the
> reason is structural rather than an implementation difficulty.
>
> Plan 56 reads arity off the signature's arrow spine, and partial application is therefore a
> **positional prefix** of that spine. A defaulted parameter has no position in it: `f a b` with a
> third parameter defaulted is either a saturated call or a one-of-three partial application, and
> nothing in the source text distinguishes them. Choosing is an inference, which doc 03 §2 clause
> 1 forbids — and the choice is not academic, because one reading produces a value and the other a
> closure.
>
> The only resolutions are worse than the decline. Making a defaulted parameter unreachable by
> currying means the arrow spine no longer states the arity, which is the property plan 56 §1.2
> calls its *"load-bearing precondition"*. Making it reachable means a defaulted call and an
> under-applied one are spelled the same.
>
> **Labelled arguments (doc 03 §8.2) remain the answer to the readability motivation**, and they
> compose with currying precisely because they are erased positionally rather than filled in.
>
> The same §8.7 reason is also what plan 85 §9 decision 1 cites against lowering a curried
> function into a *chain* of functions — `fn add(x) -> impl Fn(u64) -> u64` fabricates the same
> API surface, and worse, because the emitted `fn` no longer matches the signature the author
> wrote. One reason, two constructs, and the venue stays closed for both.

## Declined because of scope

| Construct | From | Reason |
|---|---|---|
| Bracket type arguments | Emerald | requires translating every type expression, where doc 03 §14.1 carries a type verbatim |
| A Voltaic standard library or prelude | — | doc 02 §1.1. Voltaic has crates.io; Emerald spent 112 of its 203 plans building a standard library, and that is the cost this refusal avoids |

---

## Reopenable

**One** entry carries a venue rather than a permanent decline. There were two
until plan 86 lifted the second, which is what a venue is for:

| Construct | Venue |
|---|---|
| `with` as a functional-update *expression* | doc 03 §9.1's spread entry covers the ground; revisit if it proves insufficient |

| Lifted | By | On |
|---|---|---|
| General monadic comprehension | plan 86, `do .method` | the block modifier, which plan 60 had already made a type-free discrimination |

---

## Not declined — merely absent

## Declined on a compiled measurement

One entry, and it is the only decline in this document whose reason is a thing that was built
and run rather than a thing that was argued.

### Turbofish — `name::<T> (args)`

Plan 62 (`docs/56 - 62 - Ascription Completion.md`) §1.4. Plan 52 made `::<T>` *spellable* by
giving `::` a single meaning, and this row declined to spell it, because the case that motivated
wanting it turns out not to need it.

The hard case was reading an `id_effect` capability, where the type appears nowhere in the
expression:

```
limits : Effect<Arc<Thresholds>, Never, GateEnv>
limits = need () |> .effect
```

`need::<Thresholds> ()` is `RDX0103`, so the working spelling was a lambda parameter annotation.
Verified by compiling the hand-written Rust against `id_effect 0.13.0`: **inference reaches it
from the signature alone.** `need().effect()` resolves `T = Thresholds` from the declared return
type, and so do `need().map(|t: Arc<Thresholds>| …)` and a fully unannotated variant. All three
compiled and ran.

So the declared type is sufficient wherever it is written down, and plan 62's job was to make it
writable everywhere rather than to add a second mechanism. **One way to pin a type is better
than two**, and the one that composes with the signature line is the one the language already
has.

**Venue if it is ever wanted:** a position where neither a signature nor an ascription can
reach. Plan 62 §6 disclosed that such a position would overturn this, and none had been found
when the row shipped. One candidate has since been named — a builder method whose only argument
is a type, such as a schema-typed tool registration — and it is a candidate rather than a
counterexample until someone measures whether an ascription on the builder reaches it.

---

Worth stating, because the difference is the whole point of this document being separate from
`docs/GRAMMAR.md`:

- **Macro invocation** is `UNREACHABLE`, not declined. Doc 18 §7 retired `!` as a suffix and
  nothing has replaced it as a spelling; a row that proposes one is welcome. The fixture is
  `tests/unreachable/macro_call.sdx`.
- **Macro definition** is **both**, and is the one construct in that position: doc 03 §23 item
  3 declines it explicitly — *"accept permanently that Voltaic macros require inline Rust"* —
  and it is also unreachable. It appears in both documents, deliberately.
- **`as` casts, `unsafe trait`, `unsafe impl` and the FFI export direction** are gaps nobody
  decided on. They are in `docs/GRAMMAR.md`, not here.
