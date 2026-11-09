# Plan 18 — Attributes

Plan ID: `attributes-passthrough`
Depends on: 02 (`lexical-surface`), 16 (`module-scope-bindings`)
Milestone: 4 — *specification-correct*
Status: `done`

---

## 1. What this plan is

`#[…]` and `#![…]`, copied through.

Doc 03 §13.2 **closes doc 02 §10.2's critical gap**:

> Any `#[...]` or `#![...]` line is copied through verbatim, so every present and future
> Rust attribute is reachable with no per-attribute work and interoperability (doc 01 §26)
> becomes unconditional.

Doc 18 §5.1's table keeps `#[attr]` as one of the **two surviving meanings of `#`** — the
other being string interpolation — and points at §13.2 for the lowering, which is "verbatim".

### 1.1 Why this row is `partial` and not `not-started`

Doc 04's row 18 names two things: the `deriving` clause and the verbatim passthrough.
`deriving` shipped with doc 18 §2.4 — it is a clause of the declaration and lowers to
`#[derive(…)]`, which §13.2 is careful to call *"a native spelling for the dominant case,
not a replacement"*. This plan is the replacement's absence: everything that is not
`derive`.

### 1.2 Why this is one leaf and the lexer already did the hard part

Plan 02 lexes the construct: `TokenKind::Attr { text, inner }`, with the text *"stored
excluding `#[` and `]`"* and `inner` distinguishing `#![…]`. There is even a diagnostic for
the unterminated case, `RDX0007`. What is missing is a parser that accepts the token and an
emitter that writes it back.

So the shape of the work is: decide **what an attribute attaches to**, and attach it.

### 1.3 Attributes are trivia, not items — and why that is the whole design

An attribute is not an item. `#[inline]` does not stand alone; it modifies the declaration
below it, and `#![no_std]` modifies the file. Redox already has a mechanism for *"text that
belongs to a construct without being one"* — `Trivia`, which plan 02 built for comments and
which doc 02 §10.1 requires to survive into the `.xrs`.

Attributes join it, which buys three things that would otherwise each be work:

- **Ordering with comments is preserved.** A doc comment above an attribute above an item
  is three trivia and one item, in source order, and the emitter already writes trivia in
  order. Making attributes items instead would put them in a different list and the
  interleaving would be lost.
- **Every declaration gets them at once.** `leading` is on `FnDecl`, `DataDecl`,
  `ClassDecl`, `InstanceDecl`, `ErrorsDecl`, `ConstDecl`, `ImportDecl`, `ModuleDecl` and
  `TestDecl` — nine item kinds, and none of them needs a new field.
- **Statements get them too.** `Stmt` carries `leading`, so `#[allow(…)]` on a `let` works
  without being asked for.

The cost is that an attribute inside a `class` body attaches to the member, which needs
`emit_member` to write trivia — it already does.

### 1.4 `#![…]` and where it has to go

An inner attribute applies to the enclosing item, and at file scope that means **the top of
the file**. Rust requires inner attributes to precede every other item, so a `#![no_std]`
written halfway down a Redox file would emit halfway down the `.xrs` and rustc would reject
it.

Two options, and the second ships:

1. **Gather inner attributes to the top**, as doc 03 §15's test module is gathered.
2. **Emit in place and let rustc object.**

(2), because gathering is a *reordering* and plan 14 decision 3 established that Redox does
not reorder an author's file without cause. Here there is cause only if the author wrote it
in the wrong place, which rustc says clearly — *"an inner attribute is not permitted in this
context"* — and which plan 07 reports against the `.sdx`. Moving it would hide a mistake
rather than fix one, and doc 02 §10.1's rule that trivia reaches the output unchanged is
worth more than the convenience.

---

## 2. Verified preconditions

| Claim | How checked | Result |
|---|---|---|
| The lexer produces the token | `kind.rs` | `TokenKind::Attr { text, inner }`, text excluding `#[` and `]` |
| An unterminated attribute already has a code | plan 05's catalogue | `RDX0007` `UnterminatedAttribute` |
| `#[derive(Debug)]` does not parse today | `redox build` on a two-line fixture | `RDX0103: expected a name to begin a declaration` at column 1 |
| Trivia already reaches the output in order | plan 02, doc 02 §10.1 | `emit_trivia` writes `leading` before every item and statement |
| Nine item kinds already carry `leading` | the AST | `FnDecl`, `DataDecl`, `ClassDecl`, `InstanceDecl`, `ErrorsDecl`, `ConstDecl`, `ImportDecl`, `ModuleDecl`, `TestDecl` |
| `emit_member` writes a member's trivia | plan 25 | `emit_trivia(out, &f.leading, indent)` |

---

## 3. Concrete proof this plan targets

`examples/attributes.sdx`: the five attributes §13.2's own example lists, on the
declarations they belong on, compiled by real rustc with **zero output** — which for
`#[allow(…)]` means the lint it allows must actually fire, or the attribute is unproven.

That last point is the reason this example is worth more than a golden: an attribute that
is copied through but attached to the wrong item is still valid Rust, and only its *effect*
distinguishes the two.

---

## 4. Leaves

### Leaf 1 — `leaf-attr-trivia`

`TriviaKind::Attribute` and `TriviaKind::InnerAttribute`, produced by the lexer's trivia
collector from `TokenKind::Attr`, and emitted as `#[…]` / `#![…]`.

### Leaf 2 — `leaf-proof-and-gate`

`examples/attributes.sdx`, emit tests covering an item, a statement and a class member, a
golden, and the gate.

---

## 5. Acceptance criteria

1. `#[derive(Debug)]` above a `data` declaration emits `#[derive(Debug)]` above the
   `struct`.
2. `#![no_std]` at the top of a file emits `#![no_std]` at the top of the `.xrs`.
3. All five of §13.2's listed attributes round-trip verbatim, including
   `#[serde(rename = "id")]` with its string and its `=`.
4. An attribute above a statement emits above that statement.
5. An attribute above a `class` or `instance` member emits above that member, indented with
   it.
6. A doc comment, an attribute and an item in that order emit in that order.
7. `examples/attributes.sdx` compiles with **zero** rustc output and runs; a golden covers
   the text.
8. An attribute whose `derive` could have been `deriving` still works — §13.2's native
   spelling is *"not a replacement"*, so both reach the same Rust.

---

## 6. Out of scope, and what it discloses

- **No attribute is understood.** `#[cfg(test)]` does not make Redox skip anything, and
  `#[derive(Clone)]` does not tell Redox the type is cloneable. Verbatim means verbatim;
  every consequence is rustc's.
- **An inner attribute in the wrong place is rustc's error**, by §1.4's decision. Redox
  will not move it.
- **Attributes on a field, a variant, or a function parameter** are positions trivia does
  not currently reach: `FieldDecl` and `Constructor` have no `leading`. Doc 03 §13.2 does
  not ask for them and Rust allows them, so this is a real gap and a cheap one to close
  later — it needs a `leading` on two more nodes and nothing else.
- **`#[cfg(test)]` on a generated test module.** Plan 35 emits that attribute itself, and an
  author's `#[cfg(test)]` on a `module` is theirs; the two do not interact, and a `module`
  holding `test` blocks will carry both.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**None.** Emerald has no attribute surface, because it has no host language to pass through
to.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | Attributes are trivia | A field on each item, or an item of their own | Trivia keeps source order with comments, reaches nine item kinds and every statement at once, and is already required to survive (doc 02 §10.1) |
| 2 | An inner attribute is emitted in place | Gather inner attributes to the top of the file | Gathering is reordering, which plan 14 decision 3 declined without cause. A misplaced `#![…]` is rustc's clear error, reported against the `.sdx` by plan 07 |
| 3 | The text is stored without `#[` and `]` | Store the whole line | That is what plan 02 already does, and the brackets are the emitter's to write — an inner attribute needs different ones |
| 4 | No attribute is interpreted | Read `#[cfg(…)]` and act on it | Doc 01 §1.3 and AGENTS.md: every check rustc can make, rustc makes. A Redox that understood `cfg` would have to understand features, targets and `--cfg` flags |

---

## 10. Result

Shipped. 545 tests pass; `fmt`, `clippy -D warnings` and the gate are green. Every criterion
is met as written, which is a first in this run of plans and is entirely because §1.3's
decision was the right one.

### 10.1 Two arms and a deletion

The whole implementation is: two `TriviaKind` variants, two arms in `emit_trivia`, and the
lexer pushing trivia where it used to push a token. Nothing else changed — not an item, not
a parser, not a span map.

§1.3 predicted three things and got all three, plus one it did not claim:

| Claimed | Outcome |
|---|---|
| Ordering with comments preserved | Both orders tested, both hold |
| Nine item kinds at once | No item node gained a field |
| Statements too | `#[allow(unused)]` on a `let` works, untouched |
| — | **A class member too**, because `emit_member` already wrote its trivia at the member's indent |

### 10.2 `TokenKind::Attr` is gone

Plan 02 produced a token for the construct; this plan produces trivia instead, which left
`Attr` unreachable. It is removed rather than kept: a variant nothing produces is one every
exhaustive match carries for nothing, and `emit_expr`'s and `token_text`'s matches are
exhaustive by policy precisely so a real new variant fails to compile.

Clippy did not flag it — a `pub` enum variant in a library crate is not dead code to the
compiler — so this is the kind of thing only noticing finds.

### 10.3 The exhaustive match earned its keep again

`emit_trivia`'s own doc comment says a wildcard arm *"would silently drop a doc comment,
which is the exact defect doc 02 §10.1 exists to prevent"*. The same wildcard would have
dropped **every attribute in the file and compiled**. Adding the two kinds broke the build
until both arms existed, which is the whole argument for writing the match that way, now
demonstrated twice.

### 10.4 What the example proves that the golden cannot

§3 said an attribute attached to the wrong item is still valid Rust, so only its *effect*
distinguishes the two. `examples/attributes.sdx` leans on that deliberately:
`#[allow(non_upper_case_globals)]` sits above a lowercase `const`, and removing the
attribute makes rustc warn — which fails the example corpus, because it requires rustc to
emit nothing.

So that one line is load-bearing in both directions: it proves the attribute arrived, and it
proves it arrived *on the right declaration*. Worth noting after plan 16 §10.2 found the
corpus unable to demonstrate a lint — the same strictness that blocked that is what makes
this provable.

### 10.5 Disclosures

- **Nothing is interpreted**, as §6 said. `#[cfg(test)]` skips nothing.
- **A misplaced `#![…]` is rustc's error**, by §1.4's decision, reported against the `.sdx`
  through plan 07's map.
- **A field, a variant and a parameter still cannot carry one.** `FieldDecl` and
  `Constructor` have no `leading`, so `#[serde(rename = "id")]` on a *field* — which is
  where serde attributes usually go — is not writable. §6 called this a cheap gap; it is
  cheaper than it looked, since it needs `leading` on two nodes and two `emit_trivia` calls,
  and it is the one thing in §13.2's own example list that this plan cannot place where a
  reader would expect to put it.
