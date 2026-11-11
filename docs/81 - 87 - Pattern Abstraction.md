# Plan 87 — Pattern Abstraction

Plan ID: `pattern-abstraction`
Depends on: 60 (`bind-arrow`), 61 (`where-independence`)
Milestone: 11 — *expressive delta*
Status: `done` — leaves 1, 2, 4, 5 shipped 2026-11-11; leaf 3 (view patterns) **declined**, §10.6, and filed in `docs/DECLINED.md`

---

## 1. What this plan is

A pattern becomes nameable and reusable. Two constructs, one of which is nearly free and one of
which is not.

```
pattern Origin = Point { x = 0, y = 0 }
pattern Both a b = (Some a, Some b)

describe Origin = "the origin"
describe (Point { x, y }) = "#{x},#{y}"

case pair of
  Both l r -> l + r
  _        -> 0
```

Rust has no pattern abstraction at all. A pattern cannot be named, reused, or parameterised, and
nothing in `macro_rules!` reaches the ergonomics — which is why the RFC has stood unaccepted for
years. This is the largest piece of expressiveness in the register that costs no type knowledge
whatsoever.

### 1.1 Why it is type-free, and therefore admissible

A pattern synonym is **textual substitution at transpile time**. `Origin` at a use site becomes
`Point { x: 0, y: 0 }` in the emitted Rust, and rustc then does every check it would have done on
the written-out form — exhaustiveness, field names, types, bindings. Doc 03 §2's corollary is not
weakened in either direction: Redox checks nothing new, and rustc loses nothing.

All four clauses:

- **Type-free.** The expansion is determined by the synonym's own text. Nothing is resolved
  against a type.
- **No implied dependency.** The expansion names only what the author wrote.
- **Semantics-preserving.** The emitted pattern is the one the author would have written out,
  so ownership, borrowing and binding modes are whatever Rust makes them. No clone.
- **Reachability-positive.** New expressiveness, and no Rust construct becomes less reachable.

### 1.2 Binders come from the use site, so hygiene does not arise

`pattern Both a b = (Some a, Some b)` has `a` and `b` as **formal parameters**, not as names it
introduces. `Both l r` substitutes `l` and `r` positionally and emits `(Some(l), Some(r))`.

So no identifier is fabricated and doc 02 §6.3's `__rdx_` reservation is not needed here — which
distinguishes this row from `docs/DECLINED.md`'s refusal of `RecordWildCards`, declined because it
*"introduces names absent from the source"*. A synonym introduces none: every binder at a use site
is written at that use site.

A synonym body naming something that is *not* one of its parameters and not a path — a stray
lowercase identifier — would introduce a name. Leaf 1 refuses it, and criterion 4 asserts the
refusal.

### 1.3 View patterns are the harder half, and the lowering loses a check

`f (.trim -> "") = None` — a pattern that calls a function. Haskell's `ViewPatterns`, and the
thing that makes `case` reach shapes no pattern can describe.

Rust's `match` **cannot call a function in pattern position**, so there is no direct target. The
reachable lowering is an `if let` / `else if let` chain, which is ordinary hand-written Rust:

```rust
if let "" = x.trim() { None } else if let Some(n) = parse(x) { Some(n) } else { None }
```

That works, preserves arm order, and reads as something a person would write. It has one cost,
and it is a serious one under doc 03 §2's corollary — *"Every check rustc can make, rustc
makes"*:

**an `if let` chain is not exhaustiveness-checked, and the `match` it replaced was.** Lowering a
`case` to a chain therefore *removes* a check rustc was making. The corollary forbids Redox from
performing checks; it should equally forbid Redox from taking one away.

**The resolution, and it is syntactic rather than analytic:** a `case` containing any view pattern
**must** end in a `_` arm, and its absence is a named Redox diagnostic. That is a requirement on
the source text, not an exhaustiveness analysis — Redox counts arms, it does not reason about
coverage. rustc's check is replaced by a total fallback the author wrote, which is the only
replacement available that does not grow a checker.

§9 decision 3 records this and §6 discloses what is still lost: rustc will no longer tell the
author that a variant is unhandled, because the `_` arm handles it.

### 1.4 Scope is the file, and that boundary is already load-bearing

A synonym is usable where its declaration is visible in the file being compiled. This is the same
boundary plan 56 §1.4 drew for currying, with the same reason — plan 14 decision 4: *"No import
resolution […] Redox has no name resolution and must not grow one."*

Unlike currying, the boundary here is **safe rather than silent**. An unknown name in pattern
position is not a valid Rust pattern constructor either, so an unexpanded synonym is a rustc
error at the generated pattern rather than a wrong program. Criterion 6 asserts that.

### 1.5 It does not close the or-pattern gap

Worth stating, because a reader will expect it. `docs/GRAMMAR.md` grades `|` or-patterns
`UNREACHABLE` — doc 18 made `|` a guard — and names what is lost: *"one arm body shared across
patterns."*

A synonym names a pattern once and uses it in several arms; it does not let several patterns share
one arm body. The gap stays open and `tests/unreachable/or_pattern.sdx` stays failing.

---

## 2. Preconditions to verify

**Nothing in this section is measured yet.** The row is authored, not executed.

```bash
cargo run -p redox-cli -- build /tmp/probe.sdx && cat /tmp/probe.xrs
cargo run -p redox-cli -- check /tmp/probe.sdx
```

1. **That `pattern` is free.** A new keyword; doc 03 §4.5 holds the inventory.
2. **How a `case` arm's pattern is parsed and emitted**, and whether the pattern parser has one
   entry point or several. A synonym expansion must happen in exactly one place or it will reach
   some pattern positions and not others — equation heads, `case` arms, `if … is`, `guard … is`,
   `while … is`, and destructuring bindings are six positions and the expansion must cover all of
   them.
3. **Where the clause lowering builds its `match`.** Plan 56 §11.2 names
   `signature_from.rs:55-58` as the source of the positional binders `__rdx_a{i}`; the `match` on
   them is what leaf 3 would have to replace with a chain.
4. **Whether `redox-fmt` can re-print an expanded pattern.** Plan 56 §11.3's rule applies — *a
   pass that rewrites `Program` has two consumers and will surprise one of them* — so the
   expansion must live on the emit side, not in `parse()`, or `redox fmt` will rewrite the
   author's `Origin` into `Point { x = 0, y = 0 }`.
5. **What `->` costs inside a pattern.** The view arrow collides with the signature spine's `->`
   and with `case … of`'s arm arrow. Precondition 5 is the measurement of whether the pattern
   parser can take `->` at all, and it may force a different token for the view.

---

## 3. Concrete proof this plan targets

**`describe Origin` and `describe (Point { x, y })` both compile from one `pattern` declaration,
run, and print.**

Six cases, each failing differently:

1. **A nullary synonym** in an equation head and in a `case` arm — the two positions with
   different lowerings.
2. **A parameterised synonym**, `Both a b`, with binders used in the arm body.
3. **A synonym in all six pattern positions** of precondition 2, which is the case that catches a
   single-entry-point assumption being wrong.
4. **A synonym body with a stray identifier**, refused by a named diagnostic — §1.2.
5. **A view pattern in a `case` arm** with a `_` arm present.
6. **A view pattern in a `case` with no `_` arm**, refused by a named diagnostic — §1.3.

And two that must **not** change: every existing `case` emits a `match` and byte-identical Rust,
and `tests/unreachable/or_pattern.sdx` still fails.

---

## 4. Leaves

### Leaf 1 — `leaf-pattern-synonym`

`pattern Name = <pattern>` and `pattern Name a b = <pattern>`, declared at module scope. Textual
expansion on the emit side per precondition 4. The stray-identifier refusal is here.

### Leaf 2 — `leaf-expansion-coverage`

Expansion in all six pattern positions, with case 3 of §3 as the proof. Split from leaf 1
deliberately: precondition 2 may find six entry points rather than one, and that discovery should
not be inside the leaf that designs the construct.

### Leaf 3 — `leaf-view-pattern-in-case`

A view pattern in a `case` arm, lowered to an `if let` chain, with the mandatory `_` arm and its
diagnostic. §1.3.

### Leaf 4 — `leaf-fmt-round-trip`

`redox fmt` prints `Origin`, not its expansion. Per precondition 4 this is the plan 56 §11.3
hazard and is the leaf most likely to find the expansion in the wrong place.

### Leaf 5 — `leaf-corpus-and-docs`

`examples/pattern.sdx`. `docs/GRAMMAR.md` §1.5 gains two rows; doc 03 §6.4 — *Further pattern
forms* — gains the specification; `docs/STATUS.md` regenerated. Doc 03 §22's `RecordWildCards`
decline gains a cross-reference to §1.2, because the two constructs are distinguished by exactly
one property and a reader will conflate them.

---

## 5. Acceptance criteria

1. Each §3 case transpiles, compiles with no rustc output, runs, and its stdout is asserted.
2. A golden shows the **expanded** pattern in the `.xrs` and the **unexpanded** name in the
   `.sdx`, so the substitution is visible in the corpus rather than asserted in prose.
3. A synonym works in all six pattern positions of precondition 2, each with a running case.
4. A synonym body with a stray identifier is a named Redox diagnostic with a `tests/ui/` fixture.
5. A view pattern without a trailing `_` arm is a named Redox diagnostic with a `tests/ui/`
   fixture, per §1.3.
6. An undeclared name in pattern position produces a **rustc** error at the generated pattern,
   captured in `tests/ui/`, per §1.4 — asserted to be rustc's and not Redox's.
7. `redox fmt` round-trips a synonym declaration and every use site unexpanded.
8. Every existing `case` emits byte-identical Rust; `tests/golden/` re-runs with **no
   re-blessing** of existing files, and a `case` with no view pattern still emits a `match`.
9. `tests/unreachable/or_pattern.sdx` still fails, per §1.5.
10. All examples green; `construct_coverage` green with no new `ALLOW` entries.

---

## 6. Out of scope, and what it discloses

**View patterns in an equation head are not in this row.** `f (.trim -> "") = None` would require
replacing the clause lowering's `match` on `__rdx_a{i}` with a chain, which is the hot path for
every multi-equation function in the corpus — including every function in the self-hosted
compiler. Leaf 3 restricts the view to `case` arms, which is where it earns most of its keep, and
the equation-head position is named here as a separate future row rather than attempted and
abandoned mid-leaf.

**A `case` with a view pattern loses rustc's exhaustiveness check**, and §1.3's `_` arm is a
replacement rather than an equivalent. rustc will no longer report an unhandled variant in that
`case`, because the fallback handles it. This is the only place in the register where a Redox
construct makes rustc check *less*, and it is the reason leaf 3 is restricted and separately
gated.

**Synonyms are file-scoped** (§1.4), and widening that is plan 14 decision 4's question, refused
once already.

**No bidirectional synonyms.** Haskell's `pattern` can also be used as a *constructor*. Here it is
pattern-position only; using `Origin` as an expression is whatever Rust makes of an undeclared
name, and a bidirectional form would fabricate a constructor the author did not write — doc 03
§8.7's objection to default arguments, one construct over.

**No or-patterns** (§1.5).

**Nesting depth is unbounded and unanalysed.** A synonym whose body uses another synonym expands
recursively, and a cycle is a non-terminating expansion. Leaf 1 needs a depth cap and a
diagnostic; plan 61 §10.4 is the precedent for choosing one, having had to decide *"where
'together' is"* for a `where` group.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout asserted;
`tests/ui/` fixtures for criteria 4, 5 and 6 and for the expansion-cycle cap; goldens asserted
unchanged per criterion 8; `or_pattern` fixture still failing; `construct_coverage` green.

---

## 8. Emerald reuse

None. Emerald's pattern surface was Ruby's `case/in`, and doc 02 §4.3 records its call and match
syntax as conventional; there is no abstraction mechanism to transfer.

The transferable discipline is plan 46 §1.3's, and precondition 2 is where it bites: *"of sixteen
apparent gaps in a first careless pass, eleven were the probe's own bug."* The analogous error
here is assuming one pattern entry point because the first one found worked, shipping leaf 1, and
discovering at leaf 2 that `guard … is` takes a different path. Leaf 2 exists to find that before
leaf 1's design depends on the answer.

---

## 9. Decision log

| Decision | Chosen | Rejected | Why |
|---|---|---|---|
| 1 | Textual expansion at emit | Emit a `macro_rules!` matcher per synonym | A macro would fabricate an item the author did not write (doc 03 §8.7's objection), and `macro_rules!` in pattern position does not compose with `match` arms the way substitution does. Expansion also keeps the emitted Rust recognisable, which is the property the `.xrs` exists for |
| 2 | Expansion on the emit side | In `parse()` | Plan 56 §11.3: `redox-fmt` consumes the same `Program`, so an expansion in `parse()` makes `redox fmt` rewrite the author's source |
| 3 | A mandatory `_` arm for view patterns | Analyse coverage; or lower to a `match` with guards | Coverage analysis is an exhaustiveness checker, which doc 02 §12.4 forbids. A guarded `match` cannot bind from the view's result, which is the whole point of a view pattern. Counting arms is syntactic and is the only admissible option left |
| 4 | Views in `case` arms only | Views everywhere, including equation heads | The equation head is the hot path for every multi-equation function including the self-hosted compiler; §6 names it as its own row |
| 5 | Pattern-position only | Bidirectional synonyms | A bidirectional synonym fabricates a constructor, which is doc 03 §8.7's objection to default arguments |

---

## 10. Result

Shipped 2026-11-11, **leaves 1, 2, 4 and 5 only**. Leaf 3 — view patterns — is
**not shipped**, and §10.6 is the reason rather than a deferral.
`moon run :format :lint :test` green.

### 10.1 What shipped

Pattern synonyms, nullary and parameterised, expanded at render time. From
`tests/golden/synonym.xrs`:

| Source | Emitted |
|---|---|
| `pattern Origin = (0, 0)` + `describe Origin = …` | `match __rdx_a0 { (0, 0) => … }` |
| `pattern Both a b = (Some a, Some b)` + `Both l r ->` | `(Some(l), Some(r)) => l + r` |
| `Some Origin ->` — **nested** inside another pattern | `Some((0, 0)) => true` |

`examples/synonym.sdx` is all three, compiled, run, stdout asserted, and
`redox fmt` round-trips it byte-identically.

The nested case is the one worth naming: it is what makes the expansion a
recursive walk rather than a top-level substitution, and it reaches every
pattern that holds patterns — an `Apply`'s arguments, a tuple's and a slice's
items, an alternation's branches and an `@` binding's inner pattern.

### 10.2 Precondition 2 was wrong in a useful direction

The plan expected **six pattern entry points** in the parser and warned that
assuming one would ship leaf 1 against a wrong answer. There is **one**:
`pattern_alt`, with five callers above it.

The six are on the *other* side. `pattern_text` is the single render funnel, and
replacing `Pattern::rust_text` with it in the emitter touched six call sites —
an arm, a bind, an `if … is`, a `while … is`, a control-flow binder and the
clause binder row. So the shape of the risk was right and its location was not.

### 10.3 It found and fixed an `E0005` that had been live all along

The largest finding, and it is not about synonyms.

`signature_from` lets a clause's pattern lend its name to the parameter when the
pattern is a `Pattern::Binding`. But `pattern_app` keeps a **bare constructor**
as a `Binding` carrying the whole path — plan 81 §2.4 recorded that — so:

```
pick : Option<i32> -> i32
pick None = 0
pick (Some n) = n

->  fn pick(None: Option<i32>) -> i32      -- rustc: E0005
```

A refutable pattern in argument position. **Pre-existing, and no corpus file had
one**, because every existing clause pattern is an `Apply` or a `Record` rather
than a bare constructor. Plan 87 met it on its first line of example code,
because a synonym's name is upper case by design.

The fix needs no new information: the case of the first letter is the convention
the language already runs on — doc 18 §5.3 for a type variable, plan 68 for a
constructor pattern — so an upper-case `Binding` does not lend its name, and the
parameter takes `__rdx_a0` with the constructor moving into the `match`.
`docs/GRAMMAR.md` grades the row.

### 10.4 Reserving `pattern` was tried, measured, and reverted

`Keyword::PatternKw` lasted one build. `pattern` is a **field name in four of
this compiler's own declarations** — `Binding`, `CaseArm`, `RecordPatternField`
and `Expr::Await` — so reserving it made `redox-ast` stop transpiling with
`RDX0103` on its own source, which is as direct a measurement as this project
gets.

So `pattern` is Redox's **second contextual keyword**, after `from`. Two tokens
settle the reading at one dispatch site: the word, then an **upper-case** name,
because a use site has to read as a constructor. `pattern = Some 1` and
`pattern : Vec<Pattern>` stay an ordinary binding and signature, both of which
this compiler's source contains.

It carries `from`'s cost in `from`'s own words: a field named `pattern` is
coloured as syntax, and `COLOURED_BUT_NOT_A_KEYWORD` is where that is disclosed.

### 10.5 The grammar parsed it wrongly with zero `ERROR` nodes

`pattern Origin = (0, 0)` is a perfectly good **equation** — a name, a
parameter, a body — so before the `pattern_synonym` rule existed the grammar
parsed every synonym declaration in the example with **no `ERROR` node at all**
and the wrong shape.

That is the trap `AGENTS.md` records twice, met a third time, and the gate that
answers is the one the plan was told to prefer: `tree-sitter parse` shows
`(pattern_synonym name: … parameters: …)` only after the rule is added, where
the examples sweep was green either way. `test/corpus/binder.txt` pins it.

### 10.6 Leaf 3 is **declined**, and the reason is rustc's exhaustiveness check

§1.3 raised it and the implementation confirms it: Rust's `match` cannot call a
function in pattern position, so a view pattern's only lowering is an
`if let` / `else if let` chain — and **an `if let` chain is not
exhaustiveness-checked, where the `match` it replaces is.**

§1.3's resolution was a mandatory trailing `_` arm, counted syntactically. That
is admissible and it is still a worse language: every `case` containing one view
pattern loses rustc's *"non-exhaustive patterns"* for all of its arms, not just
the viewed one. The construct would make the compiler check less in exchange for
a convenience, which is the only place in the register that trade has been
proposed.

**So it is declined, and §9 decision 3 is withdrawn rather than deferred.** A
successor that wants view patterns needs a lowering that keeps the `match` —
which means a Rust feature that does not exist — or an argument that the
exhaustiveness loss is acceptable, which this row declines to make on its own
authority.

**Recorded in `docs/DECLINED.md` on 2026-11-11**, under *"declined because Rust
has no target for it"*, which is where a reader looking for the construct will
go. Leaving it as an unfinished leaf of a shipped row made it read as work
outstanding; it is a **decision**, and the register's other refusals live in
that document. `declined.rs` asserts the two move together.

Two narrowings were checked before filing it, and both are already expressible:
a view applied to the **scrutinee** rather than per-arm keeps the `match`, and
that is `case (len xs) of 0 -> …`, an ordinary `case` on an application; a view
returning a **bool** is a guard, which doc 18 §2.3 has. So nothing remains that
is not either a loss of checking or a spelling of something that already
works.

Everything leaf 3 would have needed is in place: the expansion walk, the render
funnel and the grammar rule all reach a `case` arm already.

### 10.7 Adding an `Item` variant costs more than an `Expr` variant

Plan 86 §10.3 wrote down the `Expr` deadlock. An `Item` variant is the same
mechanism across **six crates**: `redox-ast` twice, `redox-parser`,
`redox-emit`, `redox-driver`, `redox-fmt` and `redox-lsp` — plus a root
re-export, plus `mod` declarations for every new file, and the arms arrive one
at a time as each crate reaches the front of the build.

Two additions to plan 86's recipe, both learned here:

- **The root re-export is a separate step.** `redox_ast::PatternSynonym` is
  `E0432` until `lib.sdx`'s `export import` is bridged too, and the generated
  `lib.rs` is a different file from the one holding the variant.
- **A new module's file must be copied before its `mod` line compiles.** The
  `.xrs` sits beside the `.sdx` from any `redox build`, so the bridge is a `cp`
  and a `rustfmt`, and the next port run demolishes both.

A third thing, which is this row's own: **the keyword revert needed the
*generated* lexer patched**, because the installed binary still reserved
`pattern` and so could not transpile the `.sdx` that removes the reservation.

### 10.8 Two more things the emitter had to be told

**A synonym emits its comments and nothing else.** The first version emitted
truly nothing, which dropped the author's comments — doc 02 §10.1 makes a
comment trivia that reaches the `.xrs` — and then `emit_items`' separator, which
counts an item that wrote no text, left two blank lines where they had been.
`generated_rust_is_rustfmt_clean` caught the blanks; only reading the golden
caught the loss.

**The formatter wanted the Redox spelling.** Printing the body with
`Pattern::rust_text` wrote `pattern Both a b = (Some(a), Some(b))` — valid Rust,
not a Redox program. `redox-fmt`'s own `pattern_text` is the right printer, and
this is the **fourth** place in that crate to need a Redox spelling where the
AST held only the Rust one; `visibility_text`'s doc comment counts the first
three.

### 10.9 Corrections to this document

- §2 precondition 2 is wrong about where the six entry points are. §10.2.
- §4 leaf 3 and §9 decision 3 are **withdrawn**. §10.6.
- §1.1 and §4 assume `pattern` can be a keyword. It cannot. §10.4.
- §2 has no precondition for the cost of an `Item` variant, which was the
  largest single piece of work. §10.7.
- §6's expansion-cycle note asks for a depth cap and got one — sixteen, in
  `expanded`, with plan 61 §10.4 as the precedent for taking a number.
