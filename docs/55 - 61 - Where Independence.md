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

---

## 11. Corrections from orientation

Written after reading the code and probing it. Criterion 1's precondition is
true to the character, and the sort is cheaper than §1 fears. **But §1.3 names
the wrong blocker, §6 describes `where` as something it is not, and criterion 5
is unsound in a case that compiles today.**

### 11.1 The blocker is a type spelling, not the pattern row

§1.3 frames leaf 0's difficulty as the pattern row. That is the easy half:
`go 0 = "even"` fails at `where_locals` → `Parser::stmt`, because
`looks_like_a_binding` wants `Eq`/`Colon`/`LBrace` at the second token and sees
`Num(0)`, so `parse_expr` reads `go 0` as a call and the `=` is then unexpected.
`RDX0101`, *"expected a line break after the statement"*, at the `=`. Verified
verbatim, including the position.

The hard half is that **a `where` binding has no signature line, and a nested
Rust `fn` requires parameter and return types.** Redox has no inference and
AGENTS.md forbids it ever growing one. The multi-equation machinery all exists —
`FnDecl.clauses` is already a `Vec`, `Parser::function` already loops equations,
`emit_clauses` already lowers them to a `match` — but every bit of it is
downstream of a signature line and `parse_spine`'s arrow split.

**And the obvious spelling is already taken.** Probed:

```text
go : u64 -> &'static str
->  RDX0101: expected a line break after the statement, at the `->`
```

`go : u64` is consumed as plan 52's **ascription**. So leaf 0 is: invent a type
spelling for a local function, in a position where `:` now means something else.
Doc 03 §8.3 presumes nested `fn`s and never says how their types are written;
doc 18 §2.2 says nothing about `where` lowering at all.

That is a **surface-design decision this milestone has not budgeted**, not an
implementation leaf. Either this row depends on one, or decision 1's fallback —
implement `RDX0047`, whose number is already published in doc 03 §17 and which
has zero references under `crates/` — is the honest smaller row.

One cost to budget either way: a single pattern row produced **ten**
diagnostics, as recovery escaped the `where` block and read the remainder as
module items, including a `const k : T = …` suggestion for a pattern variable.
Leaf 0's `tests/ui` fixture captures all ten unless recovery improves first.

### 11.2 §6 is wrong about what a `where` group is

§6 says *"`where` is a declaration group and a block is a statement sequence;
this row is about the former only."* In the AST and the parser they are the
**same type**: `Clause.wheres` is a `Vec<Stmt>`, and `where_locals` pushes
`self.stmt()`. The parser says so — *"the inner block is an ordinary statement
block, so a local may be anything a statement may be."*

Measured consequences, all of which a sort can reorder:

| `where` body | Emits | Why it matters |
|---|---|---|
| `print x` then `y = x + 1` | `println!(…);` then `let y = …;` | sorting reorders **side effects** |
| `(a, b) = (x, x + 1)` | `let (a, b) = …;` | one statement binds **two** names |
| `mut n = 0` then `n := x + 1` | `let mut n = 0; n = …;` | an assignment binds **no** name, so a sort over bindings may float it anywhere |

So the row must either refuse non-binding statements in a `where` group — a
diagnostic nobody has budgeted — or accept that the sort can move a `print`.
Name it in §6 rather than leaving it to the first program that hits it.

### 11.3 Criterion 5 is unsound when a group binding shadows a parameter

§1.2 says the sort *"cannot be wrong about scope, because the group **is** the
scope."* This falsifies it, and it compiles today:

```text
h base = scaled            fn h(base: u64) -> u64 {
  where                        let scaled = base * 2;   // base = the parameter
    scaled = base * 2          let base = 100;
    base = 100                 scaled
                           }
```

`h 3` is **6** today. Treat `base` as an edge and the sort emits
`let base = 100; let scaled = base * 2;` — **200**, valid Rust, no diagnostic.
That is doc 02 §5.2's worst failure mode produced by this row itself, because
Rust `let`s are sequential: with shadowing the group is *not* the scope, and a
reference's meaning depends on its position.

Either refuse a group binding that shadows a parameter, or the sort is unsound.
Add it to §3 as a fifth case.

For contrast, the half criterion 5 gets for free: *"references to module-scope
names are not edges"* needs no work at all — a name not in the group's bound set
simply is not an edge, and nothing has to tell a module-scope name from
anything else. `Expr::Path` is a single string, so `inner::x` is never an
`Ident` and never an edge either.

### 11.4 Criterion 6 contradicts criterion 8

Criterion 6: *"the four existing `where` clauses are already ordered, so their
goldens must not move."* Criterion 8 amends `examples/generic.sdx`'s reordering
comment — and comments reach the `.xrs`. That comment is in the golden, twice.

Restate criterion 6 as **no generated code moves; comment-only golden diffs are
expected in `generic.xrs`**. Plan 52 paid exactly this tax and its §10.1 is the
precedent.

**And the corpus proves almost nothing about the sort.** The four clauses are
`largest` (1 binding), `area` (2, with one edge, in order), `sizes` (2,
independent) and `retries` (1). Three are vacuously ordered; only `area`
exercises the sort at all. A stable topological sort with ties broken by source
order preserves all four byte for byte, so §3's new example is doing all the
work.

### 11.5 Four copies of the reordering claim, and one wrong citation

§4 leaf 5 names one. There are four, plus the goldens:
`examples/generic.sdx`'s §1 header sentence *and* its in-body comment,
`redox-ast`'s `clause.rs` doc comment, and `emit_clauses.rs`'s code comment —
the last two carrying the same sentence, *"that reordering is the whole of the
lowering."*

**Criterion 8 cites the wrong document.** Doc 18 §2.2 contains no ordering
claim; it says only that *"`where` keeps one meaning"*. The section to amend is
**doc 03 §8.3**, which §1.1 already quotes verbatim and accurately.

### 11.6 Verified, and smaller corrections

- **Rust accepts mutually recursive nested `fn`s**, in either definition order —
  compiled clean. So leaf 2's sort does not even have to order a mutually
  recursive component. §2 was right to say leaf 3 must verify this rather than
  inherit it from plan 55 §1.4, which verified local items with their own
  generics and not mutual recursion.
- **Nothing emits an item inside a function body today.** `emit_member` nests
  `fn` in a `trait`/`impl`, `emit_module` nests items in a `mod`,
  `emit_tests_module` nests `#[test] fn` — none inside a block. **Plan 55 leaf 5
  is building the same carrier**, in the same milestone, and says so in its own
  §4. Coordinate them or it gets written twice.
- **§6's nested-`where` paragraph describes a construct that does not parse.**
  `Keyword::Where` is matched in exactly one place in the workspace, and
  `where_locals` attaches to a `Clause`, never to a `Stmt`. A nested `where` is
  `RDX0102`. There are no nested groups to sort independently.
- **§5's criteria are numbered 1, 2, 2, 3, 4, 5, 6, 7, 8** — nine items, two
  labelled 2, and §7 and §4 then refer to "criterion 6" meaning the sixth label.
  Renumber.
- §1 and §2's example writes `area w = scaled` / `base = w * w`; the corpus is
  `area w h = scaled` / `base = w * h`. Harmless, except that the §2 block is
  presented as a measurement.
- **Claim a count, not a number.** `RDX0111` is the last taken and `RDX0112` is
  free — and **plan 55 claims no code at all**, despite needing at least three.
  Plans 56 and 57 both still say *"next free is `RDX0110`"*, which plans 52 and
  53 took while those sentences stood. If decision 1's fallback is taken,
  `RDX0047` also needs adding to `code.rs`'s `PUBLISHED` list, which
  `every_reserved_code_is_one_doc_03_published` enforces.
- Criterion 7 is free: `docs/GRAMMAR.md` grades *"`where` clauses"* as Rust's
  **bound** clause and notes that *"Redox's own `where` introduces local
  bindings and is a different construct"*, so Redox's `where` has no graded row.
  It stops being free if leaf 5 adds one for the multi-equation form — which it
  should.
- §2 cites commit `194fdf7`, 24 commits behind and predating the type colon.
- **Unsettled, and worth a test:** `redox-fmt` emits from the same `Program`, so
  plan 56 §11.3's warning applies to this row's sort as well — if the sort
  rewrites the tree rather than ordering at emission, `redox fmt` reorders the
  author's `where` group. `formatting_preserves_meaning` is the gate that would
  catch it.
