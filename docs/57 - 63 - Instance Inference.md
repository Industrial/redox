# Plan 63 — Instance Inference

Plan ID: `instance-inference`
Depends on: 52 (`type-colon`), 62 (`ascription-completion`)
Milestone: 9 — *surface coherence*
Status: `planned`

---

## 1. What this plan is

An `instance` method may omit its signature. The class supplies it, with the trait's own
parameters substituted for the instance head's arguments.

```
class Money                     instance Money Entry
  cents_of : &self -> i64         cents_of self = self.cents
  pretty : &self -> String        -- signature taken from the class, unchanged
  pretty self = format_cents (self.cents_of ())
```

**Amended by plan 67 — see §11.6.** This example used to read `class Money t` with
`cents_of : &t -> i64`, and the substitution it describes used to include the *implementing*
type. Plan 67 marked the receiver and gave the head's slot to the trait, so the substitution for
the implementing type is now the **identity** and what is left to substitute is only the trait's
real parameters — which this row's §1.4 already refuses to resolve when there is more than one.

This is the one row in the milestone that adds machinery Redox has deliberately not had, and it
should be read with that in mind. §1.3 is the argument; §6 is the honest cost.

### 1.1 The deviation is recorded in doc 18 §2.5, which already weighed this row and refused it

Three corrections to how this row's first draft cited its own subject, because the citation is
load-bearing.

**It is doc 18 §2.5, not §11.5.** §11.5 is *"Three findings from converting rather than
designing"* and is about the `--1` comment rule, hand-written position literals, and
`no_literal_expectations.rs`. `examples/class.sdx` carries the same wrong pointer — *"Doc 18
§11.5 records the deviation"* — which this row inherited from the corpus. Correcting the corpus
comment is leaf 6's, and it is a second instance of plan 51's subject found by hand.

**§2.5's example does restate the signature**, and §2.5 says why it does:

> **An instance method restates its signature**, and the line above is a correction to this
> document's first draft, which omitted it. Rust's `impl` restates it too; Redox would have to
> resolve the class to copy one from it, and AGENTS.md forbids growing any name resolution. The
> cost is one line per method and the alternative was a feature that could not be built.

So the omitting form was a draft, corrected deliberately — which makes this row a **reversal of a
considered decision** rather than the closing of an oversight.

**And AGENTS.md does not contain a name-resolution prohibition.** Its rule is about *checks*:
*"Redox has no type checker, no borrow checker and no exhaustiveness checker, and must never grow
one."* Doc 18 §2.5 and `examples/class.sdx` both attribute a resolution ban to it. That
attribution is the thing §1.3 examines, and it is the only part of §2.5's reasoning this row
disputes — *"a feature that could not be built"* is the claim being tested.

### 1.2 What the restatement costs, measured

**Recounted 2026-11-10, and the first count was wrong at the source rather than merely stale.**
This row reported one file. `examples/local.sdx` has had a `class` and an `instance` inside a
function body since plan 55 leaf 5, so the number was already wrong when it was written — and
plan 67 §1.3 copied it forward before anyone checked, which is how a measurement in one document
became a claim in two. Plan 67's own `examples/receiver.sdx` then added a third.

| | Count |
|---|---|
| Files with a `class` or `instance` block | **3** — `examples/class.sdx`, `examples/local.sdx` (inside a function body), `examples/receiver.sdx` |
| `instance` bodies across them | **7** |
| Method signature lines restated in an `instance` | **13** |
| …of which this row can elide | **12** — `instance Square`'s `perimeter` is inherent, so there is no class to take a signature from |

Twelve lines across three files is still not, by itself, a crisis. What the recount shows that
the first count could not is the **shape** of the cost: it tripled in a day without the construct
becoming any more popular, because a restatement is one line *per member per instance* —
`instance Tally Counter` carries six of them on its own, for a class with six members. That
scales with every class anyone writes, which is a better argument for this row than "three lines"
ever was. Two more things make it worth a row.

**The restatement can drift.** A signature that disagrees with its class is a rustc error about a
trait impl — correct, and arriving in generated Rust, about a mismatch Redox could see.

**It is the last place the surface lies.** After plans 52 through 62, `instance` is the only
construct whose Haskell form means something Redox does not do. Doc 18 §2.5 shows one thing and
the language requires another, and the corpus comment exists to explain the difference.

### 1.3 The resolution needed is narrow, and the boundary is assertable

AGENTS.md's rule is about **checks**, quoted in §1.1. Doc 18 §2.5's phrasing is about **name
resolution**, which is a different thing, is not in AGENTS.md, and which this row needs a little
of.

What it needs:

1. given `instance Money Entry`, find the declaration `class Money t`;
2. read the method's declared signature from it;
3. substitute `t := Entry`.

What it does not need: types, inference, trait solving, coherence checking, or knowledge of
whether `Entry` satisfies anything. **rustc does all of that**, and a wrong substitution is a
rustc error exactly as a wrong restatement is today.

The scope boundary is the same one plan 56 §1.4 drew for arity, in its corrected form:
**classes declared in the file being compiled.** Plan 14's `module` loads nothing — it emits
`mod name;` and plan 14 decision 4 refuses import resolution outright — so there is no
cross-file story to inherit. A class that cannot be found is a named diagnostic asking for the
signature, not a silent pass-through, so the feature degrades to today's behaviour rather than
to a rustc error.

That boundary is assertable, and criterion 6 asserts it, because a resolver that quietly widens
is how a project acquires the thing AGENTS.md forbids.

### 1.4 Multi-variable classes keep restating

A class with one type variable needs one substitution. A class with more — `class Convert a b` —
needs to know which instance argument binds which variable, and in which order, which is where a
local substitution becomes a resolution problem with genuine choices in it.

So: **omission is available when the class declares exactly one type variable**, and anything
more restates, with a diagnostic saying why. That covers every class in the corpus and the
overwhelming majority of real ones, and it keeps the mechanism a substitution rather than a
solver.

---

## 2. Verified preconditions

Measured 2026-11-10, against `redox` at `194fdf7`.

**The restatement is required.** `examples/class.sdx`, in today's `::` spelling:

```
instance Shape Square
  const SIDES :: u32 = 4
  area :: &Square -> f64
  area self = self.side * self.side
```

Omitting the signature is `RDX0101: expected '::' after the name, to begin the signature`,
pointing at the equation — the parser demands a signature line, so there is no form for a bare
equation inside an `instance` to take.

**Inherent `instance` blocks exist and have no class.** `examples/class.sdx`'s `instance Square`
with `perimeter :: &Self -> f64` — no class name, so nothing to infer from, and `Self` rather
than the type name. These must keep restating and the diagnostic must not suggest otherwise.

**Default methods already carry a signature in the class.** `describe :: &t -> &'static str` with
`describe self = "a shape"` in the class body — so the class is already the place a signature is
written, and this row reads what is there rather than needing a new declaration form.

**`const` members restate too.** `const SIDES :: u32 = 4` against the class's
`const SIDES :: u32`. Same mechanism, and in scope — §4 leaf 3.

**Every class in the corpus is single-variable.** There is exactly one, `class Shape t`, so
§1.4's limit is currently invisible and will be met by the first `class Convert a b` anyone
writes.

---

## 3. Concrete proof this plan targets

**`examples/class.sdx` loses three signature lines and compiles to byte-identical Rust.** The
emitted `impl` must still restate, because Rust's does; the *source* stops having to. A golden
diff of zero is the proof that this is a surface change.

Five more:

1. **A substituted method.** `area self = …` in `instance Shape Square`, with the emitted
   signature `fn area(&self) -> f64` matching the hand-restated version exactly.
2. **A default method overridden with no signature**, which must take the class's and not
   silently get a different one.
3. **A `const` member.** §2's `SIDES`.
4. **An inherent `instance`**, which must still restate, with a diagnostic that says *why* —
   there is no class — rather than repeating the one-variable rule.
5. **A two-variable class.** §1.4's refusal, as a `tests/ui/` fixture.

And one negative: **a restated signature that disagrees with the class is a Redox diagnostic**,
not a rustc trait error. That is leaf 4 and it is the half of this row that pays off even for
authors who keep restating.

---

## 4. Leaves

### Leaf 0 — `leaf-member-boundary`

Inside a `class` or `instance` body, an `Ident` not followed by `:` whose text **differs** from
the member currently being read starts a **new member**; the same name is the next clause. That
is `parser.rs`'s name-mismatch guard, and it is one line.

Added by §11.1, which found the blocker that stops an elided signature parsing at all and which
§4's original leaves did not contain: `Parser::function`'s clause loop treats any `Ident` not
followed by `:` as a continuation equation of the member it is already reading, so an elided
member collides with doc 18 §2.3's multi-clause function grammar and the name-mismatch guard
fires — *"RDX0103: this equation defines `tag`, but the signature above it is for `area`"*. So
§2's *"the parser demands a signature line"* is half the story; an omitted signature on the
*first* member does give `RDX0101`, which is why the simpler reading looked complete.

Numbered 0 rather than inserted, so leaves 1–6 keep the numbers §5 and §9 cite. It is **costed
and landed before leaves 1–3**, because until it exists there is nothing for the resolver to be
asked.

### Leaf 1 — `leaf-class-lookup`

Find a class declaration by name **within the file being compiled**. The narrowest possible
resolver: one name, one kind of declaration, one file. Returns not-found rather than guessing —
including for a class in a sibling file, which §1.3 states is out of scope rather than a bug.

Amended from *"one name, one kind of declaration, one file, no scopes to walk"*, which §11.3
found false: `Item::Module` nests items and is *"the only item that contains items"*, so a class
inside `module tests` is not in Rust scope for an instance outside it. **The choice is open and
owed** — §11.3 states it as a decision this leaf has to take and does not take it: top-level
items only, or walk the item tree.

### Leaf 2 — `leaf-signature-substitution`

Read the method's signature from the class and substitute the single type variable. The
substitution is textual over the verbatim type (doc 03 §14.1), which is what makes it a
substitution and not a type operation.

**What is copied is the class's already-rendered form, not §1's `t := Entry`**, which §11.2 is
the finding for. Restated after plan 67: `receiver` has dropped the marked parameter's type, so
`area : &self -> f64` renders `fn area(&self) -> f64` — and the instance's restated
`area : &self -> f64` renders the **same bytes by construction**, because both now spell the
receiver the same way. Before plan 67 the two agreed by coincidence: the class's `&t` had been
substituted to `&Self` and the instance's `&Square` had been dropped as a receiver, and they met
in the middle. For `instance Shape Square` the parser still computes `var = None` and never calls
`substitute_var` at all.

§11.2's bound survives with a new mechanism: a method mentioning a **trait parameter** in a
non-receiver position — `combine : &self -> t -> t` in `class Combine t` — renders `T` where the
restated form renders the concrete type, which is valid Rust and different bytes. So byte-identity
holds for `examples/class.sdx` rather than in general, exactly as §11.2 said, and §1.4's
single-variable restriction is what keeps the general case out of scope.

**Where the pass runs is open and owed.** §11.3 found that `receiver`, `substitute_var` and
`generalise` all run **inside** `Parser::function`, during the single pass, while order
independence is already real — an `instance` may precede its `class` in one file and compiles
today, emitting `impl Shape for Square` before `trait Shape`. So this leaf needs either a
pre-pass that collects classes before any body is parsed, or a post-parse pass that re-runs all
three. §4 described neither, and §11.3 calls it the second-largest undisclosed cost after
§11.1; it states the two options without choosing.

### Leaf 3 — `leaf-const-members`

The same for `const` members, which declare a type and no parameters.

### Leaf 4 — `leaf-mismatch-diagnostic`

A restated signature compared against the class's, with a named code on disagreement. Independent
of omission and useful without it — this is the leaf that would be worth shipping even if §1.3's
argument had failed.

### Leaf 5 — `leaf-refusals`

Three diagnostics: class not found, more than one type variable, inherent `instance`. Each with a
`tests/ui/` fixture and each saying something different, because a reader who omits a signature
needs to know which of the three they hit.

### Leaf 6 — `leaf-corpus-and-docs`

`examples/class.sdx`'s three elidable restatements removed. **Doc 18 §2.5**'s deviation marked
resolved with its reasoning preserved — and `examples/class.sdx`'s comment amended twice: once
for the resolution, and once because it cites §11.5 where it means §2.5 (§1.1).
`docs/STATUS.md` updated.

The stale-citation fix is **three sites, not two**, which §11.5 is the finding for: §1.1 names
doc 18 §2.5 and `examples/class.sdx`, and the third is
`crates/redox-emit/src/emit/fn_call_width.rs`. All three point at a section that is *"Three
findings from converting rather than designing"* and records none of what they claim.

*"`docs/GRAMMAR.md` regraded"* is withdrawn, which §11.5 is also the finding for: Traits, Impls,
and Associated types and constants are all already `SUGARED` and all already cite
`examples/class.sdx`, so nothing changes grade.

---

## 5. Acceptance criteria

1. An `instance` method of a one-variable class may omit its signature, and the emitted `impl`'s
   **Rust-code region** is byte-identical to the restated version — asserted by a golden diff
   confined to that region.

   Amended from *"the emitted `impl` is byte-identical to the restated version — asserted by a
   golden diff of zero"*, which §11.2 found cannot hold alongside leaf 6: `tests/golden/class.xrs`
   carries the example's comments — its first nine lines are `examples/class.sdx`'s verbatim —
   and leaf 6 amends one of them, twice, so the *file* cannot be byte-identical. §11.2's own
   remedy is the split taken here, and it calls it the stronger claim because it is the one a
   reader can check: the Rust code is identical and the comment region changes deliberately.

   Two further bounds from §11.2, both carried into leaf 2: byte-identity holds for
   `examples/class.sdx` rather than in general — `combine : &t -> t -> t` renders `Self` where
   the restated form renders `Square`, valid Rust and different bytes — and what makes it hold is
   copying the class's **already-rendered** signature, not §1's advertised `t := Entry`, since
   for `instance Shape Square` the parser computes `var = None` and never calls `substitute_var`.

   And this criterion rests on **leaf 0**: §11.1 found that an elided member currently parses as
   a *clause of the member above it*, so until the name-mismatch guard distinguishes them there
   is no form for an omitted signature to take.
2. A `const` member may omit its type, at **both** sites that demand it.

   Amended from *"a `const` member may omit its type"*, which §11.5 found under-specified: §2
   quotes only the member-signature refusal and never mentions the second site this criterion
   must change, `class_member.rs`'s *"expected `:` after the constant's name"*.
3. A default method overridden without a signature takes the class's.
4. A restated signature disagreeing with the class is a named Redox diagnostic, with the class's
   and the instance's forms both shown.
5. Class-not-found, multi-variable and inherent-`instance` are three distinct diagnostics with
   three `tests/ui/` fixtures — which with criterion 4's mismatch code makes **four** new codes
   for this row.

   The count is stated and the next-free number deliberately is not, which §11.5 is the finding
   for: `RDX0111` is the last taken, two sibling plans still say *"next free is `RDX0110`"* and
   both are wrong, and a number written in a plan doc goes stale the moment a sibling row lands.
6. **The resolver's scope is asserted**: a class in a module that `module` does not load is
   not-found, and the test states that boundary explicitly so a future widening is a deliberate
   change to a failing test.

   §11.4 supplies a stronger reason for that boundary than §1.3's scope discipline, and it is
   the one to cite: **the compile cache is keyed on *that file's* source bytes plus the compiler
   fingerprint**, so a cross-file resolver would make an instance's output depend on a sibling
   file that does not invalidate its cache entry. Widening the boundary is a **correctness bug**,
   not merely scope creep. Supporting facts, both verified in §11.4: a bodiless `module` has
   `items: None` — *"`mod name;` and `mod name {}` are different Rust"* — and `compile_dir` loops
   `compile_file` per file, so there is no structure in which two files' items coexist.
7. Every example green, no rustc output, stdout unchanged; `examples/class.sdx`'s golden diff
   is confined to its **comment region**, and its Rust-code region is unchanged.

   Amended from *"`examples/class.sdx`'s golden diff is **empty**"*, per §11.2 and criterion 1:
   leaf 6 amends a comment the golden carries verbatim, so an empty diff and leaf 6 are
   incompatible.
8. `construct_coverage` green.
9. **Doc 18 §2.5** marked resolved, citing this row and preserving its original reasoning; and
   the stale `§11.5` pointer corrected to `§2.5` at **all three** sites.

   Amended from *"`examples/class.sdx`'s §11.5 pointer corrected to §2.5"*, which §11.5 found
   to be an undercount: the third site is `crates/redox-emit/src/emit/fn_call_width.rs`.

---

## 6. Out of scope, and what it discloses

**This row adds a name resolver, and that is a scope expansion.** It is the only one in the
milestone and it should be read as a genuine cost rather than a technicality. The mitigations are
that it resolves one kind of name, that it cannot check anything, that its boundary is asserted
by criterion 6, and that not-found degrades to today's behaviour. The risk is the ordinary one:
a resolver that exists gets used, and the next row wanting one will cite this plan rather than
argue from scratch.

If that is judged too expensive, **leaf 4 is separable and carries most of the value** — the
mismatch diagnostic needs the same lookup and delivers the drift-catching benefit without
changing the surface. Shipping leaf 4 alone is a coherent outcome and §9 records it as the
fallback.

**Multi-variable classes keep restating.** §1.4. Every class in the corpus is single-variable, so
this limit is currently invisible, and it will be met by the first `class Convert a b` anyone
writes.

**Inherent `instance` blocks keep restating, permanently.** There is no class to read. That is
not a limitation to be lifted later; it is what an inherent impl is.

**The emitted `impl` still restates.** Rust requires it. So this row narrows the
source-to-generated gap's *contents* while widening its *shape* — one source line becomes two
generated ones. Doc 04's second ordering commitment applies, and leaf 2 must keep the span of the
instance method's name pointing at the source the author wrote.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout unchanged;
`examples/class.sdx`'s golden diff empty; three `tests/ui/` fixtures from leaf 5;
`construct_coverage` green.

---

## 8. Emerald reuse

Negative, and specific. Emerald grew a symbol table and doc 02 §12.4 declines a type checker
*because* of what that cost — and doc 03 §13.3 declines transpile-time evaluation citing
Emerald's plan 61 needing a step-limit flag. The pattern in both is a mechanism that started
narrow and was asked to do more.

This row is the test of whether that pattern can be resisted deliberately. Criterion 6 — assert
the resolver's boundary, so widening it breaks a test — is the mechanism, and it is borrowed from
plan 46's rule that an `UNREACHABLE` fixture must still **fail**, so the test breaks when a gap
closes.

---

## 9. Decision log

1. **Infer the signature, reversing doc 18 §2.5.** That section calls the alternative *"a
   feature that could not be built"*, which is the claim §1.3 disputes: the resolution needed is
   narrower than the ban it was measured against, and the ban is not in AGENTS.md.
2. **Single type variable only.** §1.4. More than one turns a substitution into a solver with
   choices in it.
3. **Not-found degrades to today's behaviour.** A diagnostic asking for the signature, rather
   than a guess or a rustc error.
4. **Leaf 4 is independent and is the fallback.** §6. If the resolver is judged too expensive,
   the mismatch diagnostic alone is worth the lookup and changes no surface.
5. **Assert the scope boundary as a test, not a convention.** Plan 46's discipline: a boundary
   that is only written down drifts; one that is asserted fails loudly when crossed.
6. **Land last among the type-level rows.** It depends on plan 52's token and plan 62's
   ascription being settled, and it is the row most likely to be cut — so it should be the one
   whose removal costs the least.

---

## 10. Result

**Done 2026-11-10.** An `instance` method omits its signature and the class supplies it, for one
name in one file. `examples/class.sdx` loses three restatements, and **`tests/golden/class.xrs`
does not move at all** — the elided form emits byte-identical Rust to the restated one, which is
criterion 2 met exactly rather than approximately.

Plan 67 is why that identity is now structural. §11.2 recorded that before it the two forms
agreed *by coincidence* — the class's `&t` was substituted to `&Self` and the instance's
`&Square` was dropped as a receiver, and they met in the middle. Both now spell the receiver
`&self`, so they agree by construction.

### 10.1 One deviation from §11.3, and it is a narrowing stated rather than hidden

§11.3 decided a **collecting pre-pass**. What shipped collects during the parse instead: a
`class` records each member's signature head as it reads it, and an `instance` borrows from that
table. The table is read-only to every consumer, so §11.3's actual requirement — *collect and
never transform*, keeping clear of plan 61's two-consumers rule — holds.

What a pre-pass would have bought is **order independence**, and without it the row has a
narrowing: **a class must appear before the instance that elides against it.** Restating a
signature still works in any order, so no program that compiled before this row stops
compiling, and the diagnostic names what it looked for so the ordering case is distinguishable
from a missing class.

The reason the pre-pass was not built is §11.2's: the head must be captured **before**
`receiver` runs, because `receiver` drops the marked parameter's type and the spine is
unrecoverable afterwards. A pre-pass would therefore have to re-parse signature lines, which is
a second copy of the spine grammar — the drift hazard §11.3 itself cites plan 34 leaf 3 about.
Order independence is recoverable later by a pre-pass that stores heads rather than one that
re-derives them.

### 10.2 Leaf 0 was real, and it needed two members to show itself

§11.1 was right that the clause loop is the blocker, and wrong about how quickly it appears. A
**single** elided member parses without leaf 0 at all, because there is no following member for
the loop to swallow. The failure needs **two consecutive** elided members:

```text
area self = self.side * self.side
describe self = "a square"

  RDX0103: this equation defines `describe`, but the signature above it is for `area`
```

A one-member probe looked complete and was not. `examples/class.sdx` — the corpus file this row
exists to simplify — is what proved it, which is the argument for migrating the corpus inside
the row rather than after it.

### 10.3 The name is peeked, not consumed, and the plan does not mention it

`Parser::equations` reads the member's name **once per equation**, because doc 18 §2.3 makes
every equation repeat it. A member with a signature line spells its name twice; an elided one
spells it once. So consuming the name to decide the branch leaves the loop looking at `self`
where it expects `cents_of` — the first attempt earned exactly that, `RDX0103: expected a
member's name`, pointing one token past the name.

### 10.4 `Parser::function` is three methods now, rather than one plus a copy

`function`, `signature_head` and `equations`. The split is what lets an elided member reach the
clause loop without a second copy of the markers, the constraint clause and the spine — each of
which has an ordering rule that took a plan to establish. Two copies of doc 18 §2.3's clause
grammar is how that grammar would start disagreeing with itself.

### 10.5 Disclosed

**The diagnostic cascades.** An elided member with nothing to borrow reports its own error and
then the body's recovery retries and reports `expected ':' after the name`. The first message is
the useful one and the second is noise; `tests/ui/elided_without_a_class.stderr` records both
rather than pretending otherwise.

**Criterion 6's boundary is a property of the construct, not of the resolver.** An inherent
`instance` names no class, so widening the lookup — to walk `Item::Module`, say — would not help
it. The fixture says so.

---

## 11. Corrections from orientation

Written after reading the code and probing it. §1.3's argument holds and the
resolver is smaller than it looks. **But §4's leaves are missing the problem
that actually stops an elided signature parsing**, and two criteria cannot both
be met.

### 11.1 The blocker: an elided member is a *clause of the member above it*

`Parser::function`'s clause loop treats any `Ident` **not** followed by `:` as a
continuation equation of the member it is already reading, and the name-mismatch
guard is what fires:

```text
instance Shape Square
  area : &Square -> f64
  area self = self.side
  tag self = 1
->  RDX0103: this equation defines `tag`, but the signature above it is for `area`
```

So §2's *"the parser demands a signature line"* is half the story: omitting one
collides with doc 18 §2.3's **multi-clause function grammar**, and §4 contains
no disambiguation rule at all. Probed directly; an omitted signature on the
*first* member does give `RDX0101`, which is why the simpler reading looked
complete.

The fix needs no lookup and is one line. Inside a `class` or `instance` body, an
`Ident` not followed by `:` whose text **differs** from the member currently
being read starts a **new member**; the same name is the next clause. That is
`parser.rs`'s name-mismatch guard, and it should be **leaf 0** — costed and
landed before leaves 1–3, because until it exists there is nothing for the
resolver to be asked.

### 11.2 Criteria 1 and 7 cannot hold alongside leaf 6

Both say the golden diff is **empty** / byte-identical. The golden carries the
example's comments — `tests/golden/class.xrs`'s first nine lines are
`examples/class.sdx`'s verbatim — and leaf 6 amends one of them, twice. So the
*file* cannot be byte-identical.

Split the criterion: the **Rust code** region is byte-identical and the comment
region changes deliberately. That is the stronger claim anyway, because it is
the one a reader can check.

**And byte-identity holds for `examples/class.sdx` rather than in general.**
Copying the *class's parsed* signature works because `MethodContext::for_class`
has already substituted `t → Self` and `receiver` has already dropped the
receiver's type, so `area : &t -> f64` renders `fn area(&self) -> f64` —
identical to the restated `&Square -> f64`. But a method mentioning the variable
in a **non-receiver** position, `combine : &t -> t -> t`, renders `Self` where
the restated form renders `Square`: valid Rust, different bytes. This also
contradicts §1's advertised mechanism — for `instance Shape Square` the parser
computes `var = None` and never calls `substitute_var`, so byte-identity wants
the class's already-`Self` form copied, not §1's `t := Entry`.

### 11.3 The lookup cannot live where §4 implies, and leaf 1's scope claim is wrong

**Order independence is already real**: an `instance` may precede its `class` in
one file and compiles today, emitting `impl Shape for Square` before
`trait Shape`. But `receiver`, `substitute_var` and `generalise` all run
**inside** `Parser::function`, during the single pass. So leaf 2 needs either a
pre-pass that collects classes before any body is parsed, or a post-parse pass
that re-runs all three. §4 describes neither, and this is the second-largest
undisclosed cost after §11.1.

**Leaf 1's *"one file, no scopes to walk"* is false.** `Item::Module` nests
items and is *"the only item that contains items"*, so a class inside
`module tests` is not in Rust scope for an instance outside it. Leaf 1 has to
decide: top-level items only, or walk the tree.

#### Decided 2026-11-10: a collecting pre-pass, and top-level items only

**Where it runs: a pre-pass, and it collects only.** `receiver`,
`substitute_var` and `generalise` stay exactly where they are, inside
`Parser::function`; what the pre-pass adds is a read-only scan that fills the
class table before any body is parsed. The alternative — a post-parse pass
re-running all three — puts copies of three parser-internal functions outside
the parser, which is the drift hazard plan 34 leaf 3 moved the precedence table
onto `BinaryOp` to avoid, and it also re-derives the spans §6 promises to
preserve. One substitution site, called once, from a table that is populated
before it is read.

The pre-pass must **collect and not transform**. That keeps it clear of the
rule plan 61 states — *a pass that rewrites `Program` has two consumers and
will surprise one of them* — because a scan that only reads has no second
consumer to surprise.

**Leaf 1's scope: top-level items only.** It matches §1.3's narrowness, and the
failure mode is benign in a way the alternative's is not: a class the lookup
does not find degrades to *today's* behaviour, which is a program that still
compiles because rustc resolves it. So the narrow version cannot regress
anything, and widening to walk `Item::Module` with Rust's visibility rule stays
available later as a deliberate change driven by a failing test — which is a
better way to acquire a scope rule than inferring one now. Criterion 6's
assert-the-boundary discipline is the same instinct.

### 11.4 A stronger argument for criterion 6 than §1.3 makes

Criterion 6 asserts the resolver's boundary so a future widening is a deliberate
change to a failing test. There is a better reason than scope discipline for
that boundary, and the plan does not use it: **the compile cache is keyed on
*that file's* source bytes plus the compiler fingerprint.** A cross-file
resolver would make an instance's output depend on a sibling file that does not
invalidate its cache entry — so widening the boundary is a **correctness bug**,
not merely scope creep.

Supporting facts, both verified: a bodiless `module` has `items: None` —
*"`mod name;` and `mod name {}` are different Rust"* — and `compile_dir` loops
`compile_file` per file, so there is no structure in which two files' items
coexist.

### 11.5 Smaller corrections

- **Three stale `§11.5` citations, not two.** §1.1 names two; the third is
  `crates/redox-emit/src/emit/fn_call_width.rs`. All three point at a section
  that is *"Three findings from converting rather than designing"* and records
  none of what they claim. (§1.1's own correction about §11.5 is right.)
- **§2 is measured against a two-commit-old surface.** It quotes
  `const SIDES :: u32 = 4` and the message *"expected `::`"*; the corpus is
  already `:` and the message is *"expected `:` after the name, to begin the
  signature"*. The **code** is right. And §2 never mentions the *second* site
  criterion 2 must change, `class_member.rs`'s *"expected `:` after the
  constant's name"*.
- **State a count of diagnostic codes, not a number.** This row needs **four**.
  `RDX0111` is the last taken; plans 56 and 57 both still say *"next free is
  `RDX0110`"* and both are wrong, and 57 wants three. A number written in a plan
  doc goes stale the moment a sibling row lands.
- **Leaf 6's *"`docs/GRAMMAR.md` regraded"* has no row to regrade.** Traits,
  Impls, and Associated types and constants are all already `SUGARED` and all
  already cite `examples/class.sdx`. Nothing changes grade.
- **§1.2's "three lines in 703"**: `examples/*.sdx` is 1,741 lines, 783 of them
  neither blank nor comment. 703 matches neither; provenance unknown.
- **§1.2's "6 signature-bearing member lines"** is 9 across the file — 3 in the
  `class`, 6 in the instances. The table means *instance* lines and the row
  label does not say so; "4 method signatures" and "3 elidable" are right for
  instances.
- **31 examples, 30 goldens as of 2026-11-10** — 30 and 29 when this was measured. `examples/lexical.sdx` has no golden by design, and `examples.rs`'s `NOT_PARSEABLE` is what encodes that.

### 11.6 Amended by plan 67 — both §11.3 decisions survive, and one claim about them does not

Plan 67 (`marked-receiver`) landed first and touches the same lowering, so it checked this row's
settled decisions against the code rather than assuming them. Measured 2026-11-10, after the row
landed:

| §11.3 decision | Verdict |
|---|---|
| the lookup is a **collecting pre-pass**, leaving `receiver`, `substitute_var` and `generalise` where they are inside the single parse | **survives, untouched.** `generalise` is still called from `Parser::function`; `receiver` and `substitute_var` are still called from `class_member` immediately after it. Plan 67 moved no call site |
| leaf 1's scope is **top-level items only** | **survives.** And it gained a second benign case: `examples/local.sdx` declares a `class` and an `instance` inside a *function body*, which is neither top-level nor a module, and restates its signature today. Degrading to today's behaviour there is the same benign failure §11.3 argued for |

**Plan 67 §1.6's claim that `substitute_var` gets strictly *less* to do is half right, and the
half that is wrong matters for leaf 2.** Measured: `MethodContext` lost `var: Option<String>` and
gained `vars: Vec<(String, String)>`, so the substitution lost one entry and **gained a
dimension**.

- For a `class` with no parameters — `class Shape`, which is every class in the corpus bar one —
  `substitute_var` is now called **zero** times where it was called once. That is the "less".
- For `class Convert t` it is called once, as before, but with the subject `T` rather than `Self`,
  because `t` is now the *trait's* parameter and not the implementing type.
- For `class Pair t u` it is called **twice**, which is more than before — and that class head
  was unspellable before plan 67, so nothing regressed. It is also precisely the shape §1.4
  refuses to elide, which is the connection worth having written down: the row's
  single-variable restriction and the substitution's new arity are the same fact.

So leaf 2's substitution is simpler for the common case and the general case is the one §1.4
already declines. What plan 67 removed outright is the substitution *for the implementing type*:
with the receiver marked there is no class variable standing in for it, and `Self` substitutes to
itself.
