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

**Corpus scale.** 4 `where` clauses across 29 examples at the time of measuring, 31 as of 2026-11-10 (`docs/STATUS.md` is generated and authoritative) — `generic.sdx` (two), `module.sdx`,
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

A `where` entry may be a **function**, declared exactly as at module scope — a signature line,
then one or more equation rows:

```text
f n = go n
  where
    go : u64 -> u64
    go 0 = 0
    go k = k * 2
```

This is the prerequisite the row's first draft missed, and without it leaf 3 has no surface to
lower.

Restated from *"a `where` binding may carry a pattern row"*, which §11.1 found names the easy
half: the pattern row fails at `where_locals` → `Parser::stmt` because `looks_like_a_binding`
wants `Eq`/`Colon`/`LBrace` at the second token, so `go 0` parses as a call and the `=` is
unexpected — `RDX0101`. The hard half is that a `where` binding has **no signature line** while a
nested Rust `fn` needs parameter and return types, and Redox has no inference and may never grow
one.

§11.1a settles it, and there is no spelling to invent. A **signature line** is `Ident` then `:`
with no depth-zero `=` before the line break, which is exactly plan 52's `reaches_a_binding_eq`,
asserted in both directions already; an **equation** is `Ident` then a pattern then `=`, which is
`Parser::function`'s own `is_equation`. `go : u64` is consumed as plan 52's ascription today only
because `looks_like_a_binding` tests `peek_at(1) ∈ {Eq, Colon, LBrace}` and never asks the
question it already has an answer for. `Parser::function` is then reused wholesale, so
multi-equation bodies, guard ladders, `where`-within-`where` and generics arrive without being
re-implemented.

**A signature is required, and that is consistent rather than an exception.** A module-scope
`f n = n * 2` with no signature line is already `RDX0101`, *"expected `:` after the name, to
begin the signature"*, so a local function needs no new diagnostic and gets the same message —
and that requirement is precisely what makes a nested `fn`'s types available without inference.

One cost to budget: a single pattern row produced **ten** diagnostics, recovery escaping the
`where` block and reading the remainder as module items, including a `const k : T = …` suggestion
for a pattern variable. This leaf's `tests/ui` fixture captures all ten unless recovery improves
first.

### Leaf 1 — `leaf-dependency-graph`

For each binding in a `where` group, the subset of the group's own names appearing in its body.
One pass, name set known in advance. Case 4 of §3 is the test that it does not over-collect.

**A name bound in the group that also names a parameter is not a safe edge** — §11.3, which
criterion 6 carries. Under shadowing the group is not the scope, so that case has to be refused
rather than collected.

### Leaf 2 — `leaf-topological-emission`

Sort and emit in dependency order. Ties broken by source order, so the output is deterministic —
doc 04's emitter-determinism rule, which plan 04 established and which a sort could quietly
violate.

**Only the group's bindings are sorted.** Once a `where` entry can be a function (§11.1a), the
group holds two kinds of thing and only the *bindings* are `let`s whose order matters. Nested
`fn` items are order-independent in Rust — §11.6 verified it, in either definition order, so a
mutually recursive component does not have to be ordered at all — so the sort orders bindings
among themselves and leaves functions where they are. That is simpler than sorting a mixed group.

**The sort runs at emission and never rewrites `Program`** — settled in §11.6. `redox-fmt` and
`redox-emit` are two readers of one tree, and a pass that mutates the tree to serve one silently
changes what the other produces: here that is a *formatter* reordering lines its author wrote,
which is an edit and not a formatting change. Plan 56 §11.3 reaches the same rule from the other
side, and both rows should read it as one:

> **A pass that rewrites `Program` has two consumers and will surprise one of them.** Lowering
> decisions belong in the emitter, where only generated Rust can see them.

### Leaf 3 — `leaf-mutual-recursion`

A strongly-connected component of size greater than one, all of whose members take parameters, is
emitted as a group of nested `fn` items. A component whose members take none is leaf 4's error.

**Rust accepts mutually recursive nested `fn`s in either definition order** — compiled clean,
§11.6 — so §2 was right that this leaf must verify the claim rather than inherit it from plan 55
§1.4, which verified local items with their own generics and not mutual recursion.

**Nothing in the emitter emits an item inside a function body today**: `emit_member` nests `fn`
in a `trait`/`impl`, `emit_module` nests items in a `mod`, `emit_tests_module` nests `#[test] fn`,
and none of them inside a block. **Plan 55 leaf 5 is building the same carrier**, in the same
milestone and saying so in its own §4 — coordinate with it or it gets written twice (§11.6).

### Leaf 4 — `leaf-cycle-diagnostic`

The §1.4 refusal: a parameterless cycle, named, with the participating bindings listed. A
`tests/ui/` fixture. Criterion 5's refusal of a group binding that **shadows a parameter**
(§11.3) is a second named diagnostic and lands here too.

**Claim a count, not a number** — §11.6. `RDX0111` is the last taken and `RDX0112` is free, and
plan 55 claims no code at all despite needing at least three, so a number written here would be
wrong before the row runs. (Plans 56 and 57 both still say *"next free is `RDX0110`"*, which
plans 52 and 53 took while those sentences stood.) If decision 1's fallback is taken, `RDX0047`
also needs adding to `code.rs`'s `PUBLISHED` list, which
`every_reserved_code_is_one_doc_03_published` enforces.

### Leaf 5 — `leaf-corpus-and-docs`

A new example exercising reverse order and mutual recursion. The reordering claim — *"that
reordering is the whole of the lowering"* — is amended in **all four** places it is written: it
was true and this row makes it false, which is the kind of statement the corpus exists to keep
honest. **Doc 03 §8.3** updated; `docs/GRAMMAR.md` regraded.

Corrected from *"`examples/generic.sdx`'s comment … Doc 18 §2.2 and doc 03 updated"*, on two
counts, both §11.5's. The copies are four, plus the goldens: `examples/generic.sdx`'s §1 header
sentence **and** its in-body comment, `redox-ast`'s `clause.rs` doc comment, and
`emit_clauses.rs`'s code comment, the last two carrying the same sentence. And **doc 18 §2.2 is
the wrong citation** — it contains no ordering claim, saying only that *"`where` keeps one
meaning"*; the section to amend is doc 03 §8.3, which §1.1 already quotes verbatim and
accurately.

`docs/GRAMMAR.md` should also **gain a row** for the multi-equation `where` form, per §11.6:
Redox's own `where` has no graded row today, which is why criterion 8 is free, and leaf 0 gives it
one worth grading.

---

## 5. Acceptance criteria

1. A `where` entry may be a **function**, declared exactly as at module scope: a signature line
   and one or more equation rows — `go : u64 -> u64` then `go 0 = 0`. Both halves are `RDX0101`
   today.

   Restated from *"a `where` binding may carry a pattern row — `go 0 = "even"` — which is
   `RDX0101` today"*, which §11.1 found names the easy half of the blocker: the pattern row is a
   `looks_like_a_binding` lookahead, and the hard half is that a nested Rust `fn` needs parameter
   and return types a `where` binding has no line to carry. §11.1a decides the spelling — the
   module-scope one, with the signature **required** — and both discriminators already exist,
   plan 52's `reaches_a_binding_eq` and `Parser::function`'s `is_equation`. So the criterion
   asserts the whole declaration rather than the row alone.
2. A `where` group emits in dependency order regardless of source order, for groups of two and
   three with a chain. Only the group's **bindings** are ordered; a `where` entry that is a
   function emits as a nested `fn` and stays where it was written, per §11.1a.

   §11.2 found the group is **not the declaration list §1.2 and §6 describe**: `Clause.wheres` is
   a `Vec<Stmt>` and `where_locals` pushes `self.stmt()`, the parser saying so itself — *"the
   inner block is an ordinary statement block, so a local may be anything a statement may be."*
   Measured consequences, all of which a sort can reorder: `print x` before `y = x + 1` is a
   **side effect** in the group; `(a, b) = (x, x + 1)` is one statement binding **two** names;
   and `mut n = 0` with `n := x + 1` is an assignment binding **no** name, which a sort over
   bindings may float anywhere.

   So the row must either refuse non-binding statements in a `where` group — a diagnostic nobody
   has budgeted — or accept that the sort can move a `print`. §11.2 leaves that choice **open and
   owed**, asking only that it be named rather than left to the first program that hits it.
3. Ties are broken by source order and the emission is deterministic, asserted by a golden —
   and, per §11.6, `redox fmt` over a `where` group written in a deliberately **non-dependency**
   order preserves the author's order, asserted explicitly rather than left to
   `formatting_preserves_meaning`. The sort is applied while emitting the `let`s and never
   rewrites `Program`, so a sort that is correct at emission is invisible to that test — which
   means the test passing proves nothing until such a group is in the corpus.

   §11.6 also records that this list is numbered 1, 2, 2, 3, 4, 5, 6, 7, 8 — nine items, two
   labelled 2 — and asks for a renumber. The labels are left exactly as written, because §4 and
   §7 already cite them and renumbering would silently re-point those citations; the collision is
   recorded here instead.
4. Mutually recursive `where` bindings with parameters emit nested `fn` items and run.
5. A parameterless cycle is a named Redox diagnostic listing the bindings, with a `tests/ui/`
   fixture.
6. References to **module-scope** names are not graph edges, asserted by §3 case 4 producing no
   cycle — free, because a name not in the group's bound set simply is not an edge, and
   `Expr::Path` is a single string so `inner::x` is never an `Ident` and never an edge either.
   **A group binding that shadows a parameter is refused**, as a named diagnostic.

   Amended from *"references to parameters and to module-scope names are not graph edges"*, whose
   parameter half §11.3 found unsound in a case that compiles today:

   ```text
   h base = scaled            fn h(base: u64) -> u64 {
     where                        let scaled = base * 2;   // base = the parameter
       scaled = base * 2          let base = 100;
       base = 100                 scaled
                              }
   ```

   `h 3` is **6** today. Treat `base` as an edge and the sort emits
   `let base = 100; let scaled = base * 2;` — **200**, valid Rust, no diagnostic. So §1.2's *"it
   cannot be wrong about scope, because the group **is** the scope"* does not survive shadowing:
   Rust `let`s are sequential, and a reference's meaning depends on its position. That is doc 02
   §5.2's worst failure mode produced by this row itself, and refusing the shadow is the
   alternative to an unsound sort. §11.3 also asks for this as a fifth case in §3, which is still
   owed.
7. Every example green, no rustc output, stdout unchanged — **no generated code moves**, and
   comment-only golden diffs are expected in `generic.xrs`.

   Amended from *"the four existing `where` clauses are already ordered, so their goldens must
   not move"*, which §11.4 found contradicts criterion 8: comments are trivia and reach the
   `.xrs`, criterion 8 amends `examples/generic.sdx`'s reordering comment, and that comment is in
   the golden **twice**. Plan 52 paid exactly this tax and its §10.1 is the precedent.

   §11.4 also records what those four clauses prove about the sort, which is almost nothing:
   `largest` (1 binding), `area` (2, one edge, in order), `sizes` (2, independent) and `retries`
   (1) — three vacuously ordered, only `area` exercising the sort at all, and all four preserved
   byte for byte by a stable topological sort with ties broken by source order. §3's new example
   is doing all the work.
8. `construct_coverage` green — free as the row stands, per §11.6: `docs/GRAMMAR.md` grades
   *"`where` clauses"* as Rust's **bound** clause and notes that *"Redox's own `where` introduces
   local bindings and is a different construct"*, so Redox's `where` has no graded row. It stops
   being free when leaf 5 adds one for the multi-equation form — which it should.
9. All four copies of the reordering claim are amended — `examples/generic.sdx`'s §1 header
   sentence **and** its in-body comment, `redox-ast`'s `clause.rs` doc comment and
   `emit_clauses.rs`'s code comment, the last two carrying the same *"that reordering is the whole
   of the lowering"* — along with **doc 03 §8.3**.

   Amended from *"`examples/generic.sdx`'s reordering comment and doc 18 §2.2 are both
   amended"*: §11.5 counted four copies where leaf 5 named one, and found the citation wrong —
   doc 18 §2.2 contains no ordering claim, saying only that *"`where` keeps one meaning"*, while
   doc 03 §8.3 is the section this row supersedes and the one §1.1 already quotes verbatim.

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
**unchanged**, which is criterion 7; `construct_coverage` green.

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
the wrong blocker, §6 describes `where` as something it is not, and criterion 6
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

That looked like a **surface-design decision**, and it is not one. There is no
spelling to invent, because the spelling a Redox author already knows works and
the parser already owns both halves of the test.

### 11.1a Decided: a `where` entry may be a function, spelled as at module scope

```text
f n = go n
  where
    go : u64 -> u64
    go 0 = 0
    go k = k * 2
```

Identical to a module-scope declaration, which is the whole argument for it: a
reader needs no second shape, and `Parser::function` is reused wholesale so
multi-equation bodies, guard ladders, `where`-within-`where` and generics all
arrive without being re-implemented.

**Both discriminators exist already.**

A **signature line** is `Ident` then `:` with **no depth-zero `=` before the
line break**. That is exactly `reaches_a_binding_eq`, which plan 52 built and
asserted in both directions — `a_signature_line_is_not_read_as_a_typed_binding`
and `a_typed_binding_still_parses_as_a_binding`. The collision §11.1 found is
narrower than it read: `go : u64` is consumed as an *ascription* only because
`looks_like_a_binding` tests `peek_at(1) ∈ {Eq, Colon, LBrace}` and stops
there. It never asks the question it already has an answer for.

An **equation** is `Ident` then a pattern then `=`. That is
`Parser::function`'s own `is_equation`, which tests for an `Ident` *not*
followed by `:`.

**A signature is required, and that is consistent rather than an exception.**
Probed: a module-scope `f n = n * 2` with no signature line is already
`RDX0101`, *"expected `:` after the name, to begin the signature"*. So a local
function needs no new diagnostic and gets the same message — and that
requirement is precisely what makes a nested `fn`'s parameter and return types
available without the inference AGENTS.md forbids.

**What this does not settle** is §11.3's sort hazard: once a `where` entry can
be a function, the group holds two kinds of thing, and only the *bindings* are
`let`s whose order matters. Nested `fn` items are order-independent in Rust —
§11.6 verified it — so the sort should order bindings among themselves and
leave functions where they are. Leaf 2 owns that, and it is simpler than
sorting a mixed group.

Decision 1's fallback — implement `RDX0047` and stop there — is therefore no
longer the honest smaller row. It is a strictly smaller row that this one
contains: `RDX0047` is still what a forward reference between two *bindings*
should raise.

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

#### Decided 2026-11-10: a non-binding statement in a `where` group is refused

§11.2's option (a), and for a reason stronger than the coherence argument that
suggests it.

The coherence argument is sound as far as it goes: accepting that the sort may
move a `print` is exactly doc 02 §5.2's failure mode that §11.3 invokes to
justify refusing the parameter shadow, so taking it here and refusing it there
would be incoherent.

But the real point is categorical. **A `where` group is a set of definitions,
not a sequence of actions** — that is this row's entire premise, and the reason
the group can be sorted at all. An entry with an effect has no defined position
in a set, so refusing it is not a restriction added to the construct; it is a
statement of what the construct *is*. The diagnostic should say that: a `where`
entry defines a name, and an expression evaluated for its effect belongs in the
body.

This also disposes of the implicit third option — keep source order for the
whole group whenever any entry is non-binding — which would make the sort
depend on the presence of a `print`, so adding one silently changes the
generated order of everything around it.

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

For contrast, the half criterion 6 gets for free: *"references to module-scope
names are not edges"* needs no work at all — a name not in the group's bound set
simply is not an edge, and nothing has to tell a module-scope name from
anything else. `Expr::Path` is a single string, so `inner::x` is never an
`Ident` and never an edge either.

### 11.4 Criterion 6 contradicts criterion 9

Criterion 6: *"the four existing `where` clauses are already ordered, so their
goldens must not move."* Criterion 8 amends `examples/generic.sdx`'s reordering
comment — and comments reach the `.xrs`. That comment is in the golden, twice.

Restate criterion 7 as **no generated code moves; comment-only golden diffs are
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
  labelled 2, and §7 and §4 then refer to "criterion 7" meaning the sixth label.
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
- **Settled 2026-11-10: the sort orders at emission and never rewrites
  `Program`.** `redox-fmt` and `redox-emit` are two readers of one tree, and a
  pass that mutates the tree to serve one of them silently changes what the
  other produces. Here that means a *formatter* reordering lines its author
  wrote, which is not a formatting change — it is an edit, and the author did
  not ask for it.

  This is the same shape as plan 34 leaf 3's reason for moving the precedence
  table onto `BinaryOp`: the parser and the formatter both need it, so a copy
  for one would drift from the other. The generalisation, which plan 56 §11.3
  hits from the other side and which both rows should read as one rule:

  > **A pass that rewrites `Program` has two consumers and will surprise one of
  > them.** Lowering decisions belong in the emitter, where only generated Rust
  > can see them.

  **One structural exemption, which is worth knowing before applying the rule.**
  A **signature line** is already immune, and not by luck.
  `crates/redox-fmt/src/item_text/signature_line.rs` prints it as a **verbatim
  slice of the source**, because `parser::generalise` rewrites a signature's
  types *in place*: `largest : Ord t => Option<t> -> t` is stored as
  `Ord T => Option<T> -> T`, and the author's lowercase `t` is gone from the
  tree.

  The reason it cannot simply be undone is the sharp part — doc 18 §5.3 makes
  the **case of the first letter** the whole difference between a type variable
  and a concrete type, and `T` is a legal concrete name. So the information is
  not recoverable from the AST, printing a signature from the tree would rewrite
  every generic declaration in the corpus into Rust's convention, and that is a
  change to the *language's surface* rather than a formatting choice.

  So a parser-side rewrite of a signature does **not** surprise the formatter,
  which is why `redox fmt examples/class.sdx` is byte-identical today even
  though `receiver.rs` destroys `sig.params[0]` at parse time. Measured by the
  session that wrote plan 67, which needed the answer for its marked receiver,
  and verified here.

  **What would lift the exemption**, per that file's own §10 pointer: the AST
  keeping the source spelling and the emitter applying `rust_generic` when it
  writes — moving `generalise`'s third step from parse time to emit time. A row
  that does that makes every parser-side signature rewrite visible to the
  formatter at once, and must carry the receiver work with it.

  The rule above still holds for everything that is *not* passed through
  verbatim, which is every other position a lowering could touch.

  **And plan 67 found the limit, which is worth having beside the exemption.**
  That row reads a marked receiver at parse time — `area : &self -> f64` — and
  §1.7 took this exemption to mean the whole lowering could stay in the parser.
  It cannot. `emit_clauses` pairs `sig.params` with each clause's pattern row
  **by index**, so moving the receiver to the front in the parser and not moving
  the pattern row silently misaligns the `match` arms; and moving the pattern row
  too *is* visible, because `clause_text` prints patterns **from the tree** while
  `signature_line` slices the signature **from the source**. The formatter would
  then print a signature in one order above an equation in the other.

  So the exemption is exactly the signature *line* and nothing else on the same
  declaration. Plan 67 reads the marker in the parser and floats the receiver in
  `emit::signature`, four lines, which is this rule applied rather than
  overridden.

  So the dependency ordering is applied while emitting the `let`s, and the tree
  keeps the author's order. `formatting_preserves_meaning` is still the gate
  that would catch a regression, and §5 should assert the formatter case
  explicitly rather than relying on it — a sort that is correct at emission is
  invisible to that test, which means the test passing proves nothing until a
  `where` group with a deliberately non-dependency order is in the corpus.
