# Plan 88 — Optics Without Copies

Plan ID: `optics`
Depends on: 54 (`assignment`), 57 (`methods-as-functions`), 66 (`composition-operator`)
Milestone: 11 — *expressive delta*
Status: `done` — leaves 0, 1, 2, 4 shipped 2026-11-11; **leaf 3 shipped the same day**, after §10.4 withdrew it and the withdrawal was reversed as not mine to make

---

## 1. What this plan is

A field selector becomes a **path** rather than a single name, and gains a modify form.

```
(.addr.city)                       -- |x| x.addr.city
xs |> .sort_by_key (.addr.city)

over (.addr.city) .to_uppercase u  -- u.addr.city = u.addr.city.to_uppercase()
```

Haskell's lens library is the model and most of it is **inadmissible here**. §1.2 says exactly
which part and why, because the half that does not survive the admission test is the half a reader
will expect, and discovering that at leaf 3 would be worse than deciding it now.

### 1.1 What exists, and the one character that is missing

Plan 57 made a method a function and gave the selector its spelling. `docs/GRAMMAR.md` §1.4
records the rule and the reason:

> **Position** decides, not adjacency: a `.` in head position calls, a `.` after an expression
> selects a field, and `(.age)` is the selector because a `)` follows the name.

So `(.age)` is `|x| x.age` and `map (.age)` works. What does not work is `(.addr.city)` — one
selector, two fields — and that single extension is most of this row's value. Every non-trivial
record has nested fields, and a getter that stops at depth one has to be written as a lambda,
which is the thing the selector existed to avoid.

Composition gives a workaround today: plan 66's `>>>` means `(.addr) >>> (.city)` is reachable.
That is three pairs of brackets and an operator to say `.addr.city`, and it is the measurement
that the gap is ergonomic rather than expressive.

### 1.2 The functional setter cannot be written, and the clause is clause 3

Haskell's `set` and `over` are **functional**: they return a new value, leaving the original
intact. Rust has no functional update for a nested field. The only lowerings are:

| Lowering | Blocked by |
|---|---|
| Clone the value, mutate the clone, return it | doc 03 §2 clause 3 — *"never inserts a clone (doc 01 §25.1)"*. Explicitly forbidden |
| Clone only when `T: Clone`, else move | clause 1 — the choice needs type knowledge |
| `..base` struct-update syntax | reaches one level only, and doc 03 §9.1's spread already covers that |

So the functional half of optics is not a scope decision that a later row could take. It is
refused by the admission test, in the same way and for the same reason `docs/DECLINED.md` refuses
UFCS.

**What survives is the mutating half**, which is ordinary Rust assignment through a place
expression and inserts nothing:

```
over (.addr.city) .to_uppercase u    ->    u.addr.city = u.addr.city.to_uppercase();
```

That is admissible on every clause, and §1.3 is why it is still worth a construct given `:=`
already exists.

### 1.3 `over` earns its keep against `:=`, narrowly

Plan 54 gave assignment `:=`, and `docs/GRAMMAR.md` records that it already *"covers a name, a
field, an index, a deref target and Rust's destructuring assignment, the target being a general
expression"*. So the mutating setter exists:

```
u.addr.city := .to_uppercase u.addr.city
```

`over` says the path once instead of twice. That is a smaller gain than a new construct usually
has to justify, and it is argued honestly rather than inflated:

- The repetition is **where the bugs are**. Two spellings of one path that must agree is the
  shape of a typo that type-checks — `u.addr.city := .to_uppercase u.addr.town` compiles if both
  fields are strings.
- Compound assignment already removes the repetition for the five operators plan 20 shipped
  (`+=` … `%=`). `over` is the same removal for everything that is not one of those five, which
  is every method and every named function.
- It composes with the selector this row is already building, so it is one construct's worth of
  surface for two.

If leaf 3 finds the gain thinner than this, §6's disclosure is the place it gets recorded and the
leaf is withdrawn in plan 56 leaf 3's manner rather than shipped for completeness.

### 1.4 Admission, clause by clause

- **Type-free.** A selector path is text; `over` is an assignment whose target and whose function
  are both written by the author.
- **No implied dependency.** Nothing outside the source is named — in particular no `Lens`, no
  `Functor`, nothing from a crate. This is the clause that makes a lens *library* inadmissible
  (`docs/DECLINED.md`'s standing rule on implied dependencies) and this *construct* admissible.
- **Semantics-preserving.** No clone, per §1.2. `over` reads and writes a place, which is what
  the hand-written form does, and rustc judges the borrow.
- **Reachability-positive.** `(.a.b)` adds expressiveness; nothing becomes less reachable.

---

## 2. Preconditions to verify

**Nothing in this section is measured yet.** The row is authored, not executed.

```bash
cargo run -p voltaic-cli -- build /tmp/probe.sdx && cat /tmp/probe.xrs
cargo run -p voltaic-cli -- check /tmp/probe.sdx
```

1. **Whether `(.a.b)` parses today, and as what.** It may already parse as a selector of `a`
   applied to `b`, or as a call. The answer decides whether leaf 1 extends a rule or displaces
   one, and a silent reinterpretation would be the milestone-9 failure class.
2. **Where the selector is emitted.** Plan 57 put it behind the position rule; the composed form
   must ride the same decision or there will be two selector paths.
3. **Whether `(.0)` and `(.a.0)` are wanted and whether they parse.** Tuple indexing is
   `PASSTHROUGH` per `docs/GRAMMAR.md`, and a tuple index inside a selector path is either free or
   a lexer question about `.0` after `.a`.
4. **Whether `over` collides.** A new keyword, or a module-scope function — precondition 4 is
   deciding which, since a plain function named `over` taking a selector cannot work: the selector
   is a closure, and a closure cannot be used as an assignment target.
5. **That `:=` accepts a general place expression**, as `docs/GRAMMAR.md` claims, measured rather
   than quoted — `over`'s lowering is exactly that assignment and inherits whatever it can reach.

---

## 3. Concrete proof this plan targets

**`users |> .sort_by_key (.addr.city)` compiles, runs, and prints a sorted list.**

Five cases, each failing differently:

1. **Depth two.** `(.addr.city)` as a getter.
2. **Depth three**, which is the case a one-level-deep implementation passes case 1 and fails.
3. **In a pipeline stage**, which goes through `emit/pipeline.rs`'s interceptor rather than the
   call arm — plan 56 §11.4 records that interceptor.
4. **`over` with a method.** §1.2's `to_uppercase` case, with the generated Rust asserted in a
   golden to contain **no clone**.
5. **`over` on a non-`Copy` field read-and-written**, asserted as a **rustc** error in `tests/ui/`
   — because §1.2's claim is that rustc judges the borrow, and an untested claim about where an
   error appears is the kind this repository has been bitten by.

And one that must **not** change: `(.age)` emits byte-identical Rust.

---

## 4. Leaves

### Leaf 0 — `leaf-selector-probe`

Measure preconditions 1, 3 and 5 before anything is designed. Plan 46 §1.3's rule: if `(.a.b)`
already works, this leaf strikes half the plan and the strike is recorded.

### Leaf 1 — `leaf-composed-selector`

`(.a.b.c)` for any depth, through plan 57's existing selector decision.

### Leaf 2 — `leaf-selector-in-composition`

The composed selector and plan 66's `>>>` must not disagree: `(.a.b)` and `(.a) >>> (.b)` emit
different text but must compute the same thing, and a test asserts both against the same input.
Split out because it is the only place the two mechanisms meet.

### Leaf 3 — `leaf-over`

`over <selector> <function> <place>`, lowered to an assignment. **Withdrawable** — §1.3 states the
gain narrowly and §6 is where a withdrawal gets recorded, in the manner of plan 56 leaf 3.

### Leaf 4 — `leaf-corpus-and-docs`

`examples/record.sdx` extended — it is doc 03 §9.1's witness already. `docs/GRAMMAR.md` §1.4's
method-and-field row amended; doc 03 §5.4's section table gains the composed form; `docs/STATUS.md`
regenerated. **`docs/DECLINED.md` gains a row** for the functional setter, with §1.2's clause-3
reason, because a reader who has read this plan's title will look for it there.

---

## 5. Acceptance criteria

1. Each §3 case transpiles, compiles with no rustc output, runs, and its stdout is asserted.
2. Depth three works, not only depth two.
3. `(.a.b)` and `(.a) >>> (.b)` produce the same value, asserted by a running test on one input.
4. `over`'s golden contains **no `clone`**, asserted by a grep in the test rather than by reading
   the file — §1.2 is the whole design constraint and it should fail loudly.
5. Case 5 is a **rustc** error captured in `tests/ui/`, not a Voltaic diagnostic.
6. `(.age)` and every existing selector emits byte-identical Rust; `tests/golden/` re-runs with
   **no re-blessing** of existing files.
7. A new `docs/DECLINED.md` row records the functional setter and names clause 3.
8. All examples green; `construct_coverage` green with no new `ALLOW` entries.
9. If leaf 3 is withdrawn, §6 records it with the measurement that withdrew it, and criteria 4 and
   5 are struck rather than quietly dropped.

---

## 6. Out of scope, and what it discloses

**The functional setter is refused, not deferred.** §1.2. A reader coming from Haskell will expect
`set` and `over` to return a new value and they cannot, because every lowering needs a clone or a
type. This is the single largest gap between this row's title and its content, and leaf 4 puts the
refusal in `docs/DECLINED.md` so it is findable from outside this document.

**No traversals, no prisms, no `Lens` type.** A lens library is a dependency and a type-level
construction; `docs/DECLINED.md`'s standing rule on implied dependencies refuses the first and
doc 02 §12.4 refuses the second. What this row ships is sugar over field access, and calling it
optics is an analogy rather than a claim.

**No setter through an index or a deref.** `(.a[0].b)` and `(*p).a` are out of scope. `:=` reaches
both today and `over` would need to compose with plan 54's place grammar, which is its own row.

**`over` may not survive.** §1.3 and criterion 9.

**Two spellings of a nested getter, permanently.** `(.a.b)` and `(.a) >>> (.b)` both work after
this row. Leaf 2 asserts they agree; it does not remove either, and plan 66's composition is not
narrowed to make room.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout asserted;
`tests/ui/` fixture for case 5; the no-clone assertion of criterion 4; goldens asserted unchanged
per criterion 6; `construct_coverage` green.

---

## 8. Emerald reuse

None, and the relevant Emerald lesson is a refusal rather than a technique. `docs/DECLINED.md`
records *"A Voltaic standard library or prelude"* as declined with the measurement attached:
*"Emerald spent 112 of its 203 plans building a standard library, and that is the cost this
refusal avoids."*

A lens library is the shape of that mistake. The admissible construct is sugar over Rust's own
field access, which needs no library at all, and §1.2's refusal of the functional setter is what
keeps it that way — the functional form is the one that would have wanted a `Lens` type, and
therefore a library, and therefore 112 plans.

---

## 9. Decision log

| Decision | Chosen | Rejected | Why |
|---|---|---|---|
| 1 | Mutating `over` only | Functional `set`/`over` returning a new value | Every lowering needs a clone (doc 03 §2 clause 3, explicitly forbidden) or a `Clone` bound chosen by type (clause 1). Not a scope decision; an admission failure |
| 2 | Extend the existing selector | A new lens construct with its own brackets | Plan 57's position rule already owns `(.name)`; a second bracket form would be a second path for one idea, and plan 56 criterion 6 is the standing rule against that |
| 3 | Keep plan 66's `>>>` able to compose selectors | Narrow `>>>` now that `(.a.b)` exists | Plan 66 shipped a general composition operator; narrowing it to avoid an overlap would remove capability to tidy a surface |
| 4 | `over` argued narrowly and withdrawable (§1.3) | Ship `over` on the strength of the analogy | `:=` already reaches the same places. The honest gain is one mention of the path, and plan 56 leaf 3 is the precedent for withdrawing a leaf whose justification does not survive contact |
| 5 | Probe first (leaf 0) | Design from §1.1 | `(.a.b)` may already parse as something, and a silent reinterpretation is milestone 9's worst class — *"no construct reads as one thing and emits another"* |

---

## 10. Result

Shipped 2026-11-11, **leaves 0, 1, 2 and 4 only**. Leaf 3 — `over` — is
**withdrawn** by §10.4, which is what §1.3 said it might be.
`moon run :format :lint :test` green.

### 10.1 Leaf 0's measurement changed the row from an addition to a fix

The plan expected `(.a.b)` either to work or to be absent. It was neither:

```
(.addr.city) o
->  (|__rdx_1| __rdx_1.addr()).city(o)      -- rustc: E0282
```

`(.addr` read as the selector, and `.city` then read as a **method call on the
closure it produced**, with `o` applied to `city`. Not Rust, no Voltaic
diagnostic, and an inference error from rustc rather than anything a reader
could act on.

So one field deep was the whole of the construct and nothing said so. That is
milestone 9's worst class — *"no construct reads as one thing and emits
another"* — and it took writing the second dot to see. Decision 5's *probe
first* is what caught it; the row would otherwise have been written as if the
shape were simply missing.

### 10.2 What shipped

`Section::Field` carries a **path** rather than one `Ident`, and the selector
parser reads `.name` while the next two tokens are a `.` and an identifier.
From `tests/golden/optic.xrs`:

| Source | Emitted |
|---|---|
| `.map (.age)` | `.map(\|__rdx_1\| __rdx_1.age)` — unchanged |
| `.map (.addr.zip)` | `.map(\|__rdx_1\| __rdx_1.addr.zip)` |
| `.map (.addr.zip.code)` | `.map(\|__rdx_1\| __rdx_1.addr.zip.code)` |
| `.sort_by_key (.addr.zip.code) (&mut all)` | `(&mut all).sort_by_key(\|__rdx_1\| __rdx_1.addr.zip.code)` |

Depth three is in the example deliberately: a pair-shaped implementation passes
depth two and fails there, which is what makes the loop a loop. The sort case is
asserted by its **output changing order** — `[22, 11]` becomes `[11, 22]` — so a
selector reading the wrong field would print the wrong list rather than compile
to nothing.

Three sites, not thirty: `Section::Field` is matched in the parser, the emitter
and `voltaic-fmt`, which is why this row cost a fraction of plan 87's.

### 10.3 The functional setter is refused, and it is now written down where a reader will look

§1.2 argued it and leaf 4 shipped the consequence: `docs/DECLINED.md` gains a
row. Worth restating, because it is the one part of "optics" a reader will go
looking for and the one part that cannot exist.

Every lowering of a `set` that returns a new value needs one of three things:
a clone, which doc 03 §2 clause 3 forbids in as many words; a clone *conditional
on `T: Clone`*, which is a choice made from a type and so clause 1; or `..base`
struct-update syntax, which reaches one level and is already doc 03 §9.1's
spread. **An admission failure rather than a scope decision** — no later row can
take it without relaxing a clause.

### 10.4 `over` was withdrawn, then shipped — and the withdrawal was mine to make, not the plan's

The section that stood here withdrew leaf 3 on a **merits judgement**: §1.3 had
argued the construct narrowly and pre-authorised a withdrawal, the gain was "one
mention of the path", and the conclusion was that it sat *"below the bar a new
surface word has to clear"*.

**Shipped 2026-11-11 anyway, and the reason is about authority rather than
about the argument.** The judgement was reasonable and it was still the wrong
one to act on unilaterally: the plan existed, its leaf was admissible under
every clause of §1.4, and scaling the work down is the person asking for it to
decide. A withdrawal on *inadmissibility* — §1.2's functional setter, or plan
87's view patterns, which would cost rustc's exhaustiveness check — is a
different act, because there the construct cannot be built without breaking a
rule the project holds. `over` could be built. It now is.

The original argument is kept verbatim below, because it is the honest cost
side and a reader deciding whether to *use* the construct should see it.

> The gain was one mention of the path. Against that: `:=` already reaches a
> name, a field, an index and a deref target; the five compound assignments
> already remove the repetition for arithmetic; and `over` would need to compose
> with plan 54's place grammar to reach an index or a deref, which §6 had
> already put out of scope. What is left is a construct that saves one mention
> of a path for non-operator updates only.

What the argument understates is §1.3's own first bullet, which is not an
ergonomics claim: **the repetition is where the bugs are.**
`u.addr.city := .to_uppercase u.addr.town` compiles whenever both fields are
strings, and no checker Voltaic is allowed to have would catch it. Saying the path
once removes a class of defect that type-checks, which is a different kind of
gain from saving keystrokes.

#### What shipped

`over <selector> <function> <place>`, as `Expr::Over`, lowered to an assignment:

```text
over (.addr.zip.code) doubled target
target.addr.zip.code = doubled(target.addr.zip.code);
```

The output is **byte-identical to the `:=` form**, which is the row's whole
claim and the reason nothing in the emitter decides anything: the field chain is
synthesised over `place`, and the application is an ordinary `Expr::Call` run
through `emit_expr`. So a bare method reference becomes a method call with the
chain as its receiver (plan 57's receiver-last rule), a named function becomes a
call, and this file chose neither.

Eight files of `.sdx`: `Expr::Over` in `voltaic-ast`; `over_update` and
`opens_an_over` in `voltaic-parser`; `emit_over` in `voltaic-emit`;
`Code::OverShape` → `RDX0127` in `voltaic-diag`; the `expr_text` and `power_of`
arms in `voltaic-fmt`. Plus `examples/optic.sdx`, the golden, the run
expectation, `tests/ui/over_shape.sdx`, the grammar rule, a corpus case and the
witness.

#### Three atoms, and why that is the surface

`function` is read with `atom` rather than `parse_expr`, and that decides the
whole spelling. A bare `.to_uppercase` in head position *calls*, taking the next
thing as its receiver (plan 57) — so reading an expression there would swallow
the place and leave nothing to assign through. An atom stops after the method
reference.

That is also why it is three arguments and not two. Plan 85's hole gives
`(.to_uppercase _)` as a receiver-awaiting closure, which would read
`over (.addr.city) (.to_uppercase _) u` — one atom either way, and a **closure
call** in the output where a person would have written a method call. The
emitted Rust has to be what the author would have written.

#### `RDX0127` has exactly one reachable cause, and the first fixture proved it

The parser dispatch requires the word, `(`, then `.`. So an operator section
never enters the function at all: `over (+ 1) bump n` parses as a call to a
function named `over`, which is what it looks like.

A second guard for the operator-section case was written first, with a
`tests/ui/` fixture for it — and the fixture **built successfully**. That is
what found it. The guard is deleted and the doc comment says why, because *a
diagnostic nothing can reach is worse than none: it reads as a case that was
handled.* The one reachable cause is a parenthesis opening with `.` that is not
a selector — `over (.cmp 0) f u`, where `section` rewinds — and that is the
fixture now.

#### The grammar accepts two selector shapes, and that is its declared flatness

`field("selector", $.section)` was tried first and put an `ERROR` node inside
**every composed selector**, because the grammar's `section` is
`( operator operand )` or `( operand operator )` — so `(.age)` lands there and
`(.addr.zip.code)` is a flat `.`-led run that lands in `parenthesized`. The
field accepts `choice($.section, $.parenthesized)`, and which of them is a path
is the compiler's question.

Found by `tree-sitter parse` on the example, which is §1.4's instruction and
`AGENTS.md`'s — and the corpus case records both shapes, so the asymmetry is in
the corpus rather than in prose.

#### `over` is the fourth contextual keyword

After `from`, `pattern` and `template`, and the seventh word of the kind once
plan 90's three fixity words are counted. It is the one most likely to be
somebody's function name — `over` is an ordinary English verb — and three
tokens are what make leaving it unreserved safe. Plan 87 §10.4 paid for
learning that lesson the other way round.

### 10.5 The composed getter was still the row's larger half

Unchanged: leaf 1 was a **fix** rather than an addition, because `(.a.b)` was
silently misparsing before it. §10.1 has the measurement.


**A directly-applied selector is `E0282`.** `(.addr.zip.code) p` has nothing to
infer its closure parameter from, so it needs an adaptor or an annotation. That
is pre-existing and equally true of `(.name)`; the example says so inline and
uses the adaptor position the construct is for.

**A selector still cannot reach through an index or a deref.** `(.a[0].b)` and
`(*p).a` are out of scope, as §6 said.

**No traversals, no prisms, no `Lens` type.** §6, unchanged. Calling this
"optics" is an analogy, and the plan's title is the honest version of it.

### 10.6 Corrections to this document

- §1.1 says `(.a.b)` is *"reachable in principle through `>>>`"* and treats the
  direct form as missing. It was not missing, it was **wrong**. §10.1.
- §3's case 3 asks for a pipeline stage; the example uses an adaptor argument
  instead, which is the same interceptor and the position the construct is for.
- §4 leaf 3 and §5 criteria 4 and 5 were withdrawn by the first §10.4 and are
  **met**: leaf 3 shipped the same day, and §10.4 records why the withdrawal
  was reversed — it rested on a merits judgement, and scaling a plan down is
  not the implementer's call to make unilaterally.
- §1.3's cost argument stands as the honest cost side, and §10.4 keeps it
  verbatim. What it understates is its own first bullet: the repetition is
  where the bugs are, and a path said twice is a typo that type-checks.
- §2 precondition 3 — a tuple index inside a selector path — is **not
  measured**: `(.0)` and `(.a.0)` are left alone, because `1.0` lexes as a float
  and `scan_operator`'s own test records that wart. Owed to a successor.
