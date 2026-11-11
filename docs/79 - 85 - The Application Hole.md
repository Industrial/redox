# Plan 85 — The Application Hole

Plan ID: `application-hole`
Depends on: 56 (`currying`), 57 (`methods-as-functions`), 66 (`composition-operator`)
Milestone: 11 — *expressive delta*
Status: `done` — leaves 1–5 shipped 2026-11-11; §10 is the record

---

## 1. What this plan is

`_` in an argument position becomes a closure parameter. Partial application stops depending on
the callee.

```
map (String::from _)       -- |__rdx_1| String::from(__rdx_1)
filter (.contains "x" _)   -- |__rdx_1| __rdx_1.contains("x")
map (.insert k _ m)        -- |__rdx_1| m.insert(k, __rdx_1)
sort_by (compare _ _)      -- |__rdx_1, __rdx_2| compare(__rdx_1, __rdx_2)
```

Plan 56 shipped currying, and §11.1 of that row proved its boundary is not movable by effort:
arity is knowable **only** for a bare `Expr::Ident` naming a signature in the file being
compiled. This row removes the dependence on arity rather than widening what arity is known.

### 1.1 Plan 56's boundary, restated exactly

Plan 56 §11.1 is the finding, and it is unusually final:

> Under-application is detectable only where arity is known, and arity is known **only** for a
> bare `Expr::Ident` naming a signature in this file. Everything else is unknown-arity: every
> `Expr::Path` (`String::from`), every `where` local, every equation binder, every lambda
> parameter, and every bare name reached through `import std::cmp::max`.

So four of the five things a Redox program calls cannot be partially applied, and plan 56 §6
records the consequence as a reader trap rather than a limitation:

> **`String::from` is passable, not curryable.** […] A reader who has just learned that Redox
> functions curry will expect otherwise.

That trap is this row's subject. A hole gives that reader a spelling.

### 1.2 Arity becomes call-site syntactic, which is strictly stronger

Currying reads arity from the **callee's** declaration. A hole reads it from the **call site's**
own text: *n* holes make an *n*-parameter closure, and the callee is never consulted.

That is what makes it work where currying cannot. It is also strictly more expressive in a way
currying is not, and the reason is positional:

| | Abstracts over | Reaches |
|---|---|---|
| Currying | a **suffix** of the argument list | Redox functions with an in-file signature |
| A hole | **any** position, any number | every callee, including Rust paths and methods |

`.insert k _ m` is the case that makes the difference concrete. The receiver is last (plan 57),
the map is supplied, the key is supplied, and the *middle* argument is the one abstracted. No
prefix-based mechanism can express it, and nothing in the surface can today.

### 1.3 The scope of a hole is the innermost enclosing application

The decision this row lives or dies on, and the one Scala's `_` is notorious for.

**Chosen: the closure's extent is the innermost enclosing application.** `f (g _ 1) 2` is
`f(|x| g(x, 1), 2)` — the hole belongs to `g`, not to `f`. A hole therefore never escapes the
call it is written in, and a reader finds its binder by looking left to the nearest callee.

The alternative — the innermost enclosing *parenthesised* expression, which is Scala's rule —
was rejected in §9. It makes `(f _ + 1)` and `f _ + 1` mean different things, and this surface
has parenless application everywhere, so parentheses are not a reliable landmark here the way
they are in Scala.

### 1.4 It reuses plan 56's lowering path, which is a standing criterion

Plan 56 criterion 6 requires that operator sections and function partial application *"share one
lowering path"*, and plan 56 §6 records that plan 66's composition joined it: *"`emit/compose.rs`
hands its body to the same `closure` a section and a partial application use."*

A hole is the fourth caller of that function and must not become a second path. Plan 56 §11.2
also fixes the shape it must emit: **a plain closure with no `move`**, because `move` on a
capture of a borrowed argument turns rustc's borrow error into a worse move error about the same
mistake.

And the parameter family is settled: `__rdx_N`, **not** `__rdx_a{i}`, which plan 56 §11.2 records
as already taken by the clause lowering's positional binders — `|__rdx_a|` would shadow
`fn fib(__rdx_a0: u64)`.

### 1.5 `_` already has three jobs, and position decides between them

`_` is live in the surface today, and this row adds a fourth meaning rather than a first.

| Position | Meaning | Witness |
|---|---|---|
| An equation's parameter | wildcard binder | `pick a _ = a`, `examples/generic.sdx` |
| Inside a pattern | wildcard | `examples/case.sdx` |
| Inside a type | inferred type | `:: Vec<_>`, verbatim and untouched |
| **An argument of an application** | **a closure parameter** | this row |

Position deciding is the rule plan 57 already established for `.`: *"a `.` in head position
calls, a `.` after an expression selects a field, and `(.age)` is the selector because a `)`
follows the name."* The same mechanism, one token over. Precondition 2 is the measurement that it
holds.

---

## 2. Preconditions to verify

**Nothing in this section is measured yet.** The row is authored, not executed.

```bash
cargo run -p redox-cli -- build /tmp/probe.sdx && cat /tmp/probe.xrs
cargo run -p redox-cli -- check /tmp/probe.sdx
```

1. **Whether `_` parses as an expression at all today.** Expected `RDX0101`. If it parses as
   something, that something has to be named before it is displaced.
2. **That the four positions in §1.5 are distinguishable without lookahead.** The equation-head
   case is the one at risk: `f _ = e` is a wildcard parameter and `f _` in a body is a hole, and
   both are `_` directly after a name.
3. **The shared closure emitter's entry point.** Plan 56 §11.3 places the rewrite at
   `emit_with_map`/`emit_items` entry and is explicit that it must **not** be threaded through
   `emit_expr`, which has 24 importers and 36 consumers.
4. **That `f ()` is still zero arguments.** Plan 56 §11.4 records `emit/expr_list.rs:22-24`
   returning `""` for `[Expr::Unit]` — *"`f ()` is `args.len() == 1` in the AST and zero arguments
   in Rust"* — and warns that any arity pass must special-case it or mis-saturate every
   zero-argument call in the corpus.
5. **The two interceptors between `Expr::Call` and the Call arm.** Plan 56 §11.4: `emit/pipeline.rs`
   destructures a call on the right of `\|>` so it never reaches the Call arm, and the unit case
   above. A hole inside a pipeline stage goes through the first of these.
6. **Plan 56 §11.5's callee-position bug.** A synthesised lambda in callee position is emitted
   unparenthesised — `print ((+ 1) 2)` became `|__rdx_1| __rdx_1 + 1(2)`. Plan 56 §10.5 records it
   as real and fixed in the row that created synthesised lambdas; this row creates more of them
   and must confirm the fix covers them.

---

## 3. Concrete proof this plan targets

**`xs |> .iter |> .map (String::from _) |> .collect : Vec<String>` compiles, runs, and prints.**
A Rust path, partially applied, which plan 56 §6 states is impossible.

Six cases, each failing differently:

1. **A Rust path.** `String::from _` — the case plan 56 §1.5 declined.
2. **A method's middle argument.** `.insert k _ m` — §1.2's case, unreachable by any prefix rule.
3. **A method's receiver, explicitly.** `.contains "x" _`, which must emit the same closure that
   the bare `.contains "x"` section already emits. If the two differ, there are two paths and
   §1.4 is violated.
4. **Two holes.** `compare _ _` — left-to-right numbering, asserted in a golden.
5. **Nesting.** `f (g _ 1) 2` emits `f(|__rdx_1| g(__rdx_1, 1), 2)`, which is §1.3's rule and the
   case that distinguishes it from Scala's.
6. **An imported bare name.** `max _ 3` where `max` came through `import std::cmp::max` — a name
   plan 56 §11.1 lists as permanently unknown-arity.

And two that must **not** change: `pick a _ = a` still emits an underscore-prefixed parameter and
rustc's unused-variable lint stays silent, and `add 1` still curries by plan 56's path with no
hole written.

---

## 4. Leaves

### Leaf 1 — `leaf-hole-token`

`_` becomes an expression in argument position, distinguished from the three positions in §1.5.
Precondition 2 is the risk and this leaf either confirms it or reports a lookahead requirement.

### Leaf 2 — `leaf-hole-lowering`

Collect holes per application, number them left to right, emit one closure through plan 56's
shared `closure` — no `move`, `__rdx_N` parameters, per §1.4.

### Leaf 3 — `leaf-method-holes`

The same for a method application, where the receiver is the last argument (plan 57). Case 3 of
§3 is the assertion that this is not a second mechanism: `.contains "x" _` and `.contains "x"`
must emit identical text.

### Leaf 4 — `leaf-nesting-scope`

§1.3's innermost-application rule, with case 5 of §3 as its golden and a `tests/ui/` fixture for
a hole in a position that has no enclosing application.

### Leaf 5 — `leaf-corpus-and-docs`

`examples/application.sdx` extended — plan 51 created it and plan 56 leaf 6 extended it once
already. Plan 56 §6's *"`String::from` is passable, not curryable"* disclosure is **amended** to
name this row's spelling. `docs/GRAMMAR.md` §1.4 gains the hole beside method calls and function
calls; doc 03 §5.4's section table gains it beside the operator sections.

---

## 5. Acceptance criteria

1. Each §3 case transpiles, compiles with no rustc output, runs, and its stdout is asserted.
2. `.contains "x" _` and `.contains "x"` emit **byte-identical** Rust, asserted in a golden.
   This is plan 56 criterion 6 extended to the fourth caller.
3. Holes number left to right, asserted for two and for three.
4. §1.3's scope rule is asserted by case 5's golden, and a hole with no enclosing application is
   a named Redox diagnostic with a `tests/ui/` fixture.
5. The emitted closure carries **no `move`**, per plan 56 §11.2, asserted in a golden.
6. `pick a _ = a` and every other existing use of `_` emits byte-identical Rust — `tests/golden/`
   re-runs with **no re-blessing**.
7. `f ()` is still a zero-argument call, asserted against precondition 4's case.
8. A hole inside a pipeline stage works, which goes through `emit/pipeline.rs`'s interceptor
   rather than the Call arm.
9. Plan 44 gains a benchmark pair for the hole lowering, because plan 56 §6 recorded that its own
   closure allocation *"is paid silently, which is a genuine cost in a language whose zero-cost
   claim plan 44 measures by IR equality"* and then left the pair to a future row. This is that
   row.
10. All examples green; `construct_coverage` green with no new `ALLOW` entries.

---

## 6. Out of scope, and what it discloses

**It does not make Rust functions curry.** `String::from` with one argument missing and no hole
written still emits a call and rustc still answers, exactly as plan 56 §1.5 and §11.1 decided.
This row adds a spelling; it does not change the default.

**There is no diagnostic for a forgotten hole.** `map String::from` with an argument missing is
the same undecidable case plan 56 §11.1 withdrew leaf 3 over, and nothing here makes it
decidable. The hole is opt-in and its absence is invisible.

**Two spellings for a suffix partial application.** `add 1` and `add 1 _` will both work, and for
an in-file signature they are the same closure. That is a real overlap, and §9 decision 4 records
why it is accepted rather than resolved: the hole's domain is strictly larger, and forbidding it
where currying also reaches would make the rule conditional on whether a signature is visible —
which is exactly the property plan 56 §1.4 warns is not stable across files.

**Closure allocation, again.** Same cost as plan 56 §6, same mitigation, now measured by
criterion 9 rather than deferred.

**Holes in a type are not this row's.** `Vec<_>` is inside verbatim type text and stays Rust's
inferred-type `_`. The two never meet, because a type is never an argument position.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout asserted;
`tests/ui/` fixture for an unenclosed hole; goldens asserted unchanged per criterion 6 and
re-blessed only for the new examples; `benchmarks/pairs/` gains the criterion 9 pair;
`construct_coverage` green.

---

## 8. Emerald reuse

None, and plan 56 §8 already explains why: *"Emerald had no currying, no sections and no
juxtaposed application."* There is nothing to transfer.

The transferable discipline is the one plan 56 §8 names, and it applies harder here. Doc 02 §5.2:
*"a Redox expression that reads like Rust and means something different is the worst possible
failure mode for a transpiler."* A hole captured in the wrong position compiles and prints a
different number, so only doc 02 §8.3's answer catches it — *"compiled by rustc, run, and its
stdout asserted"*. Every §3 case is a running program for that reason, and case 4's numbering is
asserted in a golden because a swapped pair of holes is exactly the defect that type-checks.

---

## 9. Decision log

| Decision | Chosen | Rejected | Why |
|---|---|---|---|
| 1 | Call-site holes | **Lowering a curried function into a chain of functions** — `fn add(x: u64) -> impl Fn(u64) -> u64` | Four reasons, and the first is already project doctrine. Doc 03 §8.7 declines default arguments because *"a synthesised wrapper fabricates API surface the user never wrote, which breaks trait implementations, rustdoc and go-to-definition"* — a chain fabricates the same surface, and worse, because the emitted `fn` no longer matches the signature the author wrote. Second: every stage that captures a reference needs a synthesised lifetime, which is borrow reasoning in the emitter, and plan 56 §1.3 refused exactly that. Third: a saturated `add 1 2` becomes `add(1)(2)`, which breaks plan 44's IR-equality pairs — the standing measurement of the zero-cost claim. Fourth: a curried method changes a trait's shape, so an `instance` body would no longer match its `class`. And the gain is nil: plan 56 already makes `add 1` a value |
| 2 | Innermost **application** (§1.3) | Innermost parenthesised expression (Scala's rule) | With parenless application everywhere, parentheses are not a reliable landmark; Scala's rule would make `(f _ + 1)` and `f _ + 1` differ |
| 3 | Position decides `_`'s meaning | A distinct token for the hole | Plan 57 already established position-decides for `.` and it is the cheaper rule to learn. If precondition 2 finds a genuine ambiguity this decision reverses, and the reversal is cheap because it is one token |
| 4 | Accept the overlap with currying for suffix cases | Forbid a hole where currying reaches | The restriction would be conditional on whether a signature is visible in this file, which plan 56 §1.4 states is not stable across a `module` boundary. A rule that changes with file layout is worse than an overlap |
| 5 | Reuse plan 56's `closure` | A hole-specific emitter | Plan 56 criterion 6 is standing and plan 66 already joined the shared path. A fourth caller is the design; a fourth path is the defect |

---

## 10. Result

Shipped 2026-11-11. `moon run :format :lint :test` green, plus `redox-bench`'s
`the_lowerings_are_free` for criterion 9.

### 10.1 What shipped

`_` in an argument position is a closure parameter. Measured, from
`tests/golden/hole.xrs`:

| Source | Emitted |
|---|---|
| `.map (String::from _)` | `.map(\|__rdx_1\| String::from(__rdx_1))` |
| `.map (.trim _)` | `.map(\|__rdx_1\| __rdx_1.trim())` |
| `.insert "a" _ (&mut counts)` | `\|__rdx_1\| (&mut counts).insert("a", __rdx_1)` |
| `add _ _` | `\|__rdx_1, __rdx_2\| add(__rdx_1, __rdx_2)` |
| `twice (add _ 1) 5` | `twice(\|__rdx_1\| add(__rdx_1, 1), 5)` |
| `max _ 0` | `\|__rdx_1\| max(__rdx_1, 0)` |
| `_ := pick 1 2` | `_ = pick(1, 2);` — **unchanged** |

`examples/hole.sdx` is all seven, compiled, run, stdout asserted.

### 10.2 The implementation is a substitution and a re-entry, which is why
methods were free

§4's leaves 2 and 3 are one change. `emit_application` replaces each hole with a
generated parameter name and **re-enters itself** with the substituted
arguments, then wraps the result in `closure`. So the `$` flatten, plan 57's
receiver-last `method_ref`, the unit-argument reading and `missing_arity` all run
on the substituted call, unchanged.

That is plan 56 criterion 6's one-lowering rule honoured by *reuse* rather than
by a fourth caller of `closure` — and leaf 3 needed no code at all, because a
method's arguments go through the same list. §1.3's innermost-application scope
also falls out for free: the closure wraps the `emit_application` the hole is an
argument of, so `twice (add _ 1) 5` puts it around the inner call.

### 10.3 `_` was never `RDX0101`, and the plan's precondition 1 was wrong

Expected to be a parse error in expression position. It is not: `_` lexes as an
ordinary `Expr::Ident` whose text is `"_"` and emits verbatim, so `add _ 2`
already produced `add(_, 2)` before this row. So the row changed what a hole
**means** rather than adding one.

**And the plan did not know about the fourth position.** rustc's own message for
the old behaviour names it — *"in expressions, `_` can only be used on the
left-hand side of an assignment"* — and `_ := e` is exactly that: legitimate,
emitting Rust's discard `_ = e;`, measured clean. It is untouched because an
assignment target never reaches `emit_application`, and `examples/hole.sdx`
writes it so the distinction is in the corpus.

It was also **in no corpus file**, which is why `grammar.js` still parsed it as a
wildcard pattern followed by an `ERROR`. `docs/GRAMMAR.md`'s assignment row lists
five target shapes and did not list this one. Fixed here; graded `PASSTHROUGH`.

### 10.4 Criterion 2 was wrong, and the correction is the row's best case

Criterion 2 asked that `.contains "x" _` and `.contains "x"` emit **byte-identical**
Rust. They must not, and measuring it is what showed why:

```
.contains "x"     ->  "x".contains()          -- "x" IS the receiver
.contains "x" _   ->  |__rdx_1| __rdx_1.contains("x")
```

Plan 57's rule is that the **last written argument is the receiver**, so a
one-argument method section has its receiver supplied and nothing missing. There
is no bare-section equivalent of the hole form, which is the point rather than a
defect — and criterion 2 is amended to assert the two *differ*, with both in the
golden.

**The consequence is larger than the criterion.** *Mapping a method over a
collection was unwritable before this row.* `(.trim)` is the **selector** — a
`.name` followed by `)` selects a field — so `.map (.trim)` emitted
`|x| x.trim` and rustc answered `E0615`, *"attempted to take value of method"*.
The only previous spelling was a written lambda. `docs/GRAMMAR.md` gains a row
for it and **README.md's own table was corrected**: it claimed `.trim` alone was
`|x| x.trim()`, which was false for the whole life of the document.

### 10.5 Criterion 4's diagnostic is withdrawn, because rustc's is better

The plan asked that a hole with no enclosing application be a named Redox
diagnostic. It is not, and should not be. rustc already answers it precisely and
the message is remapped to the `.sdx` with the right span:

```
x = _
->  in expressions, `_` can only be used on the left-hand side of an assignment
```

That names the one legitimate position, which a Redox diagnostic would have had
to restate. AGENTS.md's *"every check rustc can make, rustc makes"* applies, and
this is a check rustc makes well. Criterion 4's scope rule half still holds and
is asserted by the golden.

### 10.6 The lowering is free, which plan 56 deferred

`benchmarks/pairs/hole.{sdx,rs}` is criterion 9 and discharges plan 56 §6's note
that its closure allocation *"is paid silently […] Plan 44's harness should gain
a pair for this lowering"*. `the_lowerings_are_free` passes: the hole closure and
the hand-written one produce identical IR.

The pair deliberately abstracts a **middle** argument — `sub 100 _` — because a
suffix case would have measured what plan 56 already could have.

### 10.7 What this row exposed and did not fix

**The emitter still cannot break a method chain.** `examples/hole.sdx`'s first
draft put `borrowed.into_iter().map(|__rdx_1| String::from(__rdx_1)).collect()`
on one line, which rustfmt's `chain_width` breaks — so the example splits the
statement in two and says why. Plan 51 found this and plan 58 owns it; the hole
makes a chain one closure longer and so meets it sooner.

**`_` in a type is untouched.** `Vec<_>` is inside verbatim type text and never
an argument position, so the two cannot meet.

**No diagnostic for a forgotten hole.** `map String::from` with an argument
missing is the undecidable case plan 56 §11.1 withdrew its leaf 3 over, and
nothing here makes it decidable. The hole is opt-in and its absence is invisible.

### 10.8 Corrections to this document

- §2 precondition 1 is wrong: `_` parses today. §10.3.
- §2 has no precondition for the `:=` discard, which is the position that
  mattered most. §10.3.
- §5 criterion 2 is inverted — the two forms must differ. §10.4.
- §5 criterion 4's diagnostic is withdrawn. §10.5.
- §4's leaf 3 required no code; §10.2 is why.
