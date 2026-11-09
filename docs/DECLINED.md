# Declined

What Redox will not have, and why. Doc 03 §22 is the source; this restates it for a reader who
has not read a 2,000-line surface document, and
`crates/redox-cli/tests/declined.rs` asserts every §22 construct appears here so the two
cannot drift.

**A decline is not a gap.** A gap is a Rust construct a Redox program cannot express —
`docs/GRAMMAR.md` grades those `UNREACHABLE` and `tests/unreachable/` holds them failing. A
decline is a construct Redox has decided **not to offer**, usually one no other language's
version would survive doc 03 §2's admission test. Conflating the two makes both useless, which
is why they are separate documents.

Each entry is permanent unless a venue is named.

---

## Declined because it would imply a dependency

Doc 03 §2's second admission test: a construct's lowering may name nothing outside `core`,
`alloc` and `std`. A construct that needs a crate makes every Redox program depend on that
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
| General monadic comprehension | Scala | must choose `flat_map` versus `and_then`. **Venue**: revisit only if a type-free discrimination is found |
| A Redox type checker | — | the project's load-bearing constraint (doc 02 §12.4) |

## Declined because Rust has no target for it

A lowering needs something to lower *to*. These have no Rust form, so a Redox spelling would be
a feature of the compiler rather than of the language.

| Construct | From | Reason |
|---|---|---|
| Custom operator declaration | Haskell, Scala | no Rust lowering target exists (doc 03 §14.4) |
| Symbol literals and `%i[...]` | Ruby | Rust has no symbol type (doc 03 §10.4) |
| Two-binder `and` patterns | C# | no Rust pattern form binds from both sides (doc 03 §6.2) |
| Transpile-time evaluation (`comptime`) | Zig, Emerald | requires an interpreter in the compiler (doc 03 §13.3) |

## Declined because it duplicates something Redox already has

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
| Default arguments | C#, Ruby, Python | fabricates API surface (doc 03 §8.7) |

## Declined because of scope

| Construct | From | Reason |
|---|---|---|
| Bracket type arguments | Emerald | requires translating every type expression, where doc 03 §14.1 carries a type verbatim |
| A Redox standard library or prelude | — | doc 02 §1.1. Redox has crates.io; Emerald spent 112 of its 203 plans building a standard library, and that is the cost this refusal avoids |

---

## Reopenable

Two entries carry a venue rather than a permanent decline, and doc 03 §22 names both:

| Construct | Venue |
|---|---|
| `with` as a functional-update *expression* | doc 03 §9.1's spread entry covers the ground; revisit if it proves insufficient |
| General monadic comprehension | revisit only if a type-free discrimination is found |

---

## Not declined — merely absent

Worth stating, because the difference is the whole point of this document being separate from
`docs/GRAMMAR.md`:

- **Macro invocation** is `UNREACHABLE`, not declined. Doc 18 §7 retired `!` as a suffix and
  nothing has replaced it as a spelling; a row that proposes one is welcome. The fixture is
  `tests/unreachable/macro_call.sdx`.
- **Macro definition** is **both**, and is the one construct in that position: doc 03 §23 item
  3 declines it explicitly — *"accept permanently that Redox macros require inline Rust"* —
  and it is also unreachable. It appears in both documents, deliberately.
- **`as` casts, `unsafe trait`, `unsafe impl` and the FFI export direction** are gaps nobody
  decided on. They are in `docs/GRAMMAR.md`, not here.
