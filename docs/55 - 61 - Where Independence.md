# Plan 61 — Where Independence

Plan ID: `where-independence`
Depends on: 55 (`expression-orientation`)
Milestone: 9 — *surface coherence*
Status: `planned`

---

## 1. What this plan is

`where` bindings stop caring about the order they are written in, and may be mutually recursive.

Today they are emitted in source order as a `let` sequence, so writing a binding before the one
it uses produces a rustc error about generated code:

```
area w = scaled                 fn area(w: f64) -> f64 {
  where                             let scaled = base * 2.0;      // E0425
    scaled = base * 2.0             let base = w * w;
    base   = w * w                  scaled
                                }
```

Haskell's `where` is a recursive, order-independent binding group. Redox's is an ordered `let`
list wearing the name. This row makes the name true.

### 1.1 Doc 03 §8.3 already specifies half of this row, and explicitly refuses the other half

This is the one row in the milestone that **supersedes a published specification section**, and
it has to say so before it argues anything. Doc 03 §8.3's lowering rules, in full:

> - A `where` definition that is a **function** becomes a nested `fn`, hoisted above the body —
>   so mutual recursion among them is permitted, as in Haskell.
> - A `where` definition that is a **value** becomes a `let` emitted before the body expression,
>   in written order. A forward reference between value definitions is `RDX0047`, because Rust
>   `let` bindings are ordered and synthesising a correct order would require **dependency
>   analysis this surface does not admit**.

So two corrections to how this row must be read:

**The mutual-recursion half is not a new design — it is an unimplemented specification.** §8.3
already decided nested `fn`s and already permits mutual recursion. §4 leaf 3 *implements* that;
it does not invent it.

**The order-independence half is specified as a refusal.** `RDX0047` exists on purpose, and its
stated reason is that the sort this row proposes is *"dependency analysis this surface does not
admit"*. §1.2 is the counter-argument, and it has to be good enough to overturn a published
decision rather than merely sound plausible.

And a third thing, which is why the row is reachable at all: **`RDX0047` is specified and not
implemented.** Zero references under `crates/`. So today a forward reference is not a Redox
diagnostic — it is the rustc `E0425` of §2, arriving in generated code. The status quo is
neither what §8.3 specified nor what this row proposes, and that gap is what makes the question
live rather than settled.

### 1.2 The sort is not the dependency analysis §8.3 refused

§8.3's refusal is about a general claim: synthesising an order requires knowing what names mean,
which needs resolution this surface does not have. That is true of a `let` sequence **in a block
body**, where a binding can be shadowed, imported, or a method on something.

A `where` group is not that. It is a **closed, flat list of names in one syntactic region**, and
the question asked of each binding is only: *which of the other names in this same group appear
in my body?* That is a scan over a known name set, and specifically:

- it does not resolve names **outside** the group — a reference to a parameter or a module-scope
  binding is simply not an edge, which is why §3 case 4 asserts exactly that;
- it does not need to know whether a name is a function or a value;
- it cannot be wrong about scope, because the group *is* the scope;
- it is the same argument plan 56 §1.2 made for arity, which §8.3 predates.

So the distinction this row rests on is: **§8.3 refused dependency analysis over an open name
space, and a `where` group is a closed one.** If that distinction does not hold, the row should
be refused and `RDX0047` implemented instead — which is leaf 4's fallback and §9 decision 1's
alternative.

### 1.3 Mutual recursion needs multi-equation `where` bindings, which do not parse

Haskell's `where` group is recursive, and the idiom is a pair of functions that call each other,
local to the function using them:

```
parity n = go n
  where
    go 0 = "even"
    go k = flip (k - 1)
    flip 0 = "odd"
    flip k = go (k - 1)
```

**That does not parse today**, and not because of ordering: `go 0 = "even"` inside a `where` is
`RDX0101: expected a line break after the statement` at the `=`. A `where` binding cannot carry
a **pattern row** at all, so the multi-equation form doc 03 §8.3's nested-`fn` rule presumes is
unreachable.

This row therefore has a prerequisite its own first draft missed: **leaf 0**, the multi-equation
`where` binding. Without it, "mutual recursion among them is permitted, as in Haskell" is a
sentence about a surface nobody can write.

Deferring that would mean shipping a row called *where independence* that refuses the one case
order-independence exists for — plan 20 §10.1's lesson, which this milestone also cites in
plan 54.

### 1.4 Parameterless bindings in a cycle are an error, and that is honest

A cycle among bindings that take **no** parameters cannot be emitted as nested `fn`s and called,
because each would need the other's value before either could be computed. In a strict language
that is not a program:

```
where
  a = b + 1
  b = a + 1          -- RDX: cyclic `where` bindings
```

Haskell accepts this and diverges at run time, because it is lazy. Redox is strict, so the cycle
is detectable statically and is reported statically. That is a divergence from Haskell **in
Redox's favour**, and §6 records it as such rather than as a limitation.

---

## 2. Verified preconditions

Measured 2026-11-10, against `redox` at `194fdf7`.

**Out-of-order `where` emits out-of-order `let` and fails in rustc:**

```
area w = scaled
  where
    scaled = base * 2.0
    base   = w * w
->  let scaled = base * 2.0; let base = w * w; scaled
->  rustc E0425: cannot find value `base` in this scope
```

**In-order works and is what the corpus does.** `examples/generic.sdx` has
`base = w * h` then `scaled = base * 2.0`, with a comment stating the reordering is *"the whole
of the lowering"* — which is correct about today and is the sentence this row makes obsolete.

**Rust accepts nested `fn` items**, verified for plan 55 §1.4. Mutual recursion among them is
asserted by doc 03 §8.3 rather than by plan 55, and leaf 3 must verify it rather than inherit the
claim.

**Corpus scale.** 4 `where` clauses across 29 examples — `generic.sdx` (two), `module.sdx`,
`errors.sdx`. All four are in dependency order already, so the migration is nil and the row is
purely additive to existing programs.

---

## 3. Concrete proof this plan targets

**`examples/generic.sdx`'s `area` is rewritten with its bindings in the opposite order, and
prints the same number.** The source order reverses, the output does not, and the golden shows
the emitted order was chosen by the sort.

Four more:

1. **A three-binding chain in reverse order.** `c` uses `b` uses `a`, written `c`, `b`, `a`. A
   single-pass "move it earlier" heuristic passes the two-binding case and fails this one.
2. **Mutual recursion.** §1.3's `parity`, emitted as nested `fn`s, running.
3. **A parameterless cycle.** `a = b + 1`, `b = a + 1` — a named Redox diagnostic, with a
   `tests/ui/` fixture.
4. **A `where` binding referring to a function parameter and to a module-scope constant.**
   Neither is an edge in the graph, so neither can affect the sort — asserted because a scan that
   treated every identifier as an edge would produce spurious cycles.

---

## 4. Leaves

### Leaf 0 — `leaf-multi-equation-where`

A `where` binding may carry a pattern row, so the multi-equation form §1.3 needs parses at all.
This is the prerequisite the row's first draft missed, and without it leaf 3 has no surface to
lower.

### Leaf 1 — `leaf-dependency-graph`

For each binding in a `where` group, the subset of the group's own names appearing in its body.
One pass, name set known in advance. Case 4 of §3 is the test that it does not over-collect.

### Leaf 2 — `leaf-topological-emission`

Sort and emit in dependency order. Ties broken by source order, so the output is deterministic —
doc 04's emitter-determinism rule, which plan 04 established and which a sort could quietly
violate.

### Leaf 3 — `leaf-mutual-recursion`

A strongly-connected component of size greater than one, all of whose members take parameters, is
emitted as a group of nested `fn` items. A component whose members take none is leaf 4's error.

### Leaf 4 — `leaf-cycle-diagnostic`

The §1.4 refusal: a parameterless cycle, named, with the participating bindings listed. A
`tests/ui/` fixture.

### Leaf 5 — `leaf-corpus-and-docs`

A new example exercising reverse order and mutual recursion. `examples/generic.sdx`'s comment
about reordering being *"the whole of the lowering"* is amended — it was true and this row makes
it false, which is the kind of statement the corpus exists to keep honest. Doc 18 §2.2 and doc 03
updated; `docs/GRAMMAR.md` regraded.

---

## 5. Acceptance criteria

1. A `where` binding may carry a pattern row — `go 0 = "even"` — which is `RDX0101` today.
2. A `where` group emits in dependency order regardless of source order, for groups of two and
   three with a chain.
2. Ties are broken by source order and the emission is deterministic, asserted by a golden.
3. Mutually recursive `where` bindings with parameters emit nested `fn` items and run.
4. A parameterless cycle is a named Redox diagnostic listing the bindings, with a `tests/ui/`
   fixture.
5. References to parameters and to module-scope names are not graph edges, asserted by §3 case 4
   producing no cycle.
6. All 29 examples green, no rustc output, stdout unchanged — the four existing `where` clauses
   are already ordered, so their goldens must not move.
7. `construct_coverage` green.
8. `examples/generic.sdx`'s reordering comment and doc 18 §2.2 are both amended.

---

## 6. Out of scope, and what it discloses

**Laziness is not added, and the cycle refusal is the visible consequence.** §1.4. Haskell accepts
`a = b + 1; b = a + 1` and diverges; Redox refuses it at compile time. That is a *better* outcome
and it is still a divergence, so it is written down — a reader porting Haskell will meet it.

**The sort is confined to one `where` group.** Nested `where` clauses — a `where` inside a
`where` binding's body — form separate groups, each sorted independently. Cross-group ordering is
lexical nesting and needs nothing.

**`let`-position bindings in a block are unaffected.** Plan 55 made a block a sequence of
expressions evaluated in order, and that order is semantic — a binding in a block body is
sequential and must stay so, because it can have side effects. `where` is a declaration group and
a block is a statement sequence; this row is about the former only. The two are now visibly
different constructs with different rules, which is worth stating because they look alike.

**Emission may become nested `fn`s where it was `let`s**, which widens the source-to-generated
gap for the mutual-recursion case. Doc 04's second ordering commitment cares about that; the
spans are per-binding and survive the shape change, which leaf 3 must assert.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout unchanged;
`tests/ui/` fixture for the parameterless cycle; goldens for the four existing `where` clauses
**unchanged**, which is criterion 6; `construct_coverage` green.

---

## 8. Emerald reuse

None. Emerald had no `where` and no local binding group, so there is no prior art for the sort
and none is needed — a topological sort over a closed name set is the most ordinary thing in this
milestone.

The transferable item is again plan 20 §10.1's: §1.3 keeps mutual recursion in scope *because* a
row that delivers order-independence without it has not delivered the feature. That lesson is now
cited by plans 54 and 61 in this milestone, and by plans 22 and 33 before it — four rows, which is
a reasonable sign it should be in AGENTS.md rather than in four plans' decision logs.

---

## 9. Decision log

1. **Sort, rather than implement `RDX0047`.** This is the live alternative, not a hypothetical:
   doc 03 §8.3 specifies that diagnostic and it is **unimplemented**, so "do what the
   specification says" is a smaller row than this one and a legitimate outcome. It is declined
   because it leaves `where` meaning something other than what it is named after, which is the
   milestone's subject — but if §1.2's closed-name-space argument fails, implementing `RDX0047` is
   the correct row and this one should be withdrawn rather than weakened. The third option,
   renaming `where` to `let`, resolves the divergence by giving up the construct, and the
   construct is good.
2. **Mutual recursion in scope.** §1.3.
3. **Nested `fn`s for cycles, not a `let` with a closure.** A closure cannot refer to itself
   without extra machinery; a nested `fn` can, and Rust already permits it. Verified for plan 55.
4. **A parameterless cycle is an error, not a divergence.** §1.4. Redox is strict and can tell
   statically, so it should.
5. **Ties broken by source order.** Determinism is a standing rule (plan 04), and a sort is the
   obvious place to lose it.
6. **One group at a time.** §6. Cross-group ordering is lexical nesting; treating all `where`
   bindings in a function as one group would create cycles that the nesting already resolves.

---

## 10. Result

*Not yet executed.*
