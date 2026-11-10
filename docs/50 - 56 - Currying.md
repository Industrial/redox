# Plan 56 — Currying

Plan ID: `currying`
Depends on: 52 (`type-colon`), 55 (`expression-orientation`)
Milestone: 9 — *surface coherence*
Status: `planned`

---

## 1. What this plan is

Juxtaposition starts paying for itself. An under-applied call becomes a closure instead of a
rustc error.

Redox took Haskell's application syntax — `f a b`, no parentheses, no commas — and did not take
the thing that syntax exists for. In Haskell `f a b` is two applications of one-argument
functions, which is why `map (add 1)` means something. In Redox it is one application of a
two-argument function, so `map (add 1)` emits `map(add(1))` and fails in generated Rust.

That is the single largest instance of the milestone's subject: a borrowed surface whose
semantics did not come with it.

### 1.1 The error lands in the wrong language

Measured:

```
inc_all xs = xs |> into_iter |> map (add 1) |> collect : Vec<i32>

->  xs.into_iter().map(add(1)).collect()
->  rustc: E0061: this function takes 2 arguments but 1 argument was supplied
```

Redox accepts it. rustc rejects it, pointing at a line nobody wrote, about a function whose
arity Redox knew all along. Doc 18 §4's sections — `(+ 1)`, `(1 +)`, `(> 0)` — are a patch over
exactly this hole, covering operators and leaving named functions out.

### 1.2 Arity is syntactic, and the parser already computes it

The objection to currying is that it needs to know how many arguments a function takes, and that
sounds like the symbol table AGENTS.md forbids.

It is not. A Redox signature **states** the arity: `add : i32 -> i32 -> i32` has two top-level
arrows and therefore two parameters. The signature parser already splits on top-level arrows and
already ignores arrows nested inside a type, because doc 03 §14.1 makes a type verbatim text.
Verified:

```
add2 : impl Fn(i32, i32) -> i32 -> i32
->  fn add2(f: impl Fn(i32, i32) -> i32) -> i32
```

One parameter, not three. The nested arrows are inside a verbatim type and the parser does not
count them. So arity is available today, for free, from the construct that already exists — and
this row needs no analysis Redox does not already perform.

### 1.3 Capture by move, and rustc judges the result

An under-applied call emits a `move` closure capturing the supplied arguments:

```
inc = add 1               ->  let inc = move |__rdx_1| add(1, __rdx_1);
xs |> map (add 1)         ->  xs.map(move |__rdx_1| add(1, __rdx_1))
```

Uniform, with no exceptions to remember. A capture that outlives its borrow, or a non-`Copy`
value moved where it is needed again, is a **rustc error at the generated closure** — which is
AGENTS.md's standing rule applied rather than worked around: *every check rustc can make, rustc
makes*.

The alternatives were weighed in §9. Threading a named lifetime through the generated closure
would move borrow reasoning into the emitter; restricting currying to `Copy` arguments would
make the rule conditional on types Redox cannot see.

### 1.4 Cross-module arity is the real boundary

Arity comes from a visible signature, and this row must be careful about what "visible" rests
on. **Plan 14's `module` does not load anything** — a bodiless `module name` emits `mod name;`
and reads nothing, and plan 14 decision 4 is explicit: *"No import resolution […] Redox has no
name resolution and must not grow one."* What reaches a sibling file is plan 09's
`compile_dir`, which compiles the whole directory because the directory is the unit.

So the boundary is: **currying is available where the signature is in the file being compiled.**
`compile_dir` gives a cross-file story only in the sense that every file is compiled; it does not
make one file's signatures available to another. An under-applied call to a name with no visible
signature is a named diagnostic saying so, not a silent pass-through to rustc.

That is narrower than this row's first draft claimed, and it is stated as a boundary rather than
hidden as a limitation. Widening it means cross-file signature reading, which is the name
resolution plan 14 refused — so a future row wanting it is opening that question deliberately
rather than inheriting an accident.

### 1.5 Rust paths become curried values too

`String::from` and `HashMap::new` are Rust functions, not Redox ones, so Redox has no signature
for them and cannot compute their arity. They keep their current spelling —
`HashMap::new ()`, `String::from "x"` — and additionally become usable as **function values**:

```
xs |> map String::from
```

That works because a Rust path used as a value is a `fn` item, which `map` accepts. It is not
currying; it is passing a function, and it costs nothing because it is already legal Rust. The
distinction matters and §6 records it: `String::from` is passable, not partially applicable.

---

## 2. Verified preconditions

Measured 2026-11-10, against `redox` at `194fdf7`.

**Under-application is accepted and emits a call:**

```
map (add 1)   ->  map(add(1))   ->  rustc E0061
```

**Sections already work, for operators only:** doc 18 §4's table is `(+ 1)`, `(1 +)`, `(> 0)`,
`(0 <)`, `(.name)`, `(|> name)` and `(- 1)`; `examples/section.sdx` exercises every form, with
`(10 -)`, `(> 1)` and `(1 <)` as its operands.

**`$` works** — low precedence, right-associative, respecting method chains — which matters here
because `f $ g x` and partial application interact: `$` is application, not a section, and must
keep its meaning when its left operand is under-applied.

**The signature parser ignores nested arrows**, verified in §1.2. This is the row's
load-bearing precondition and it holds today.

**A zero-argument closure cannot be written.** `\_ -> 7` emits `|_| 7` — arity one. There is no
spelling for `|| e`, which is why `Effect::retry` factories in new code are named functions
rather than closures. Plan 64 adds `\->`; this row does not depend on it but the interaction is
recorded in §6.

**Corpus scale.** 29 examples, 703 code lines; no example currently partially applies a named
function, because it cannot.

---

## 3. Concrete proof this plan targets

**`xs |> map (add 1) |> sum` compiles, runs, and prints the right number.**

Five cases, each failing differently:

1. **One of two arguments.** `inc = add 1` then `inc 5`. The simple case.
2. **One of three.** `field parts[0] "major"` awaiting the third — nested closures, which is
   where a naive single-level implementation passes case 1 and fails here.
3. **A borrowed capture.** `prefix s` where `s : &str`. Expected to be a **rustc** error about
   the generated closure's lifetime, asserted as such in `tests/ui/` — because §1.3's claim is
   that rustc judges it, and an untested claim about where an error appears is the kind this
   repository has been bitten by.
4. **Under-application inside `$`.** `print $ add 1 $ 2` — two operators whose precedence
   interacts, and the case that distinguishes "currying" from "parenthesis elision".
5. **A function whose signature is not in this file.** A named Redox diagnostic, per §1.4,
   asserted as a `tests/ui/` fixture — and asserted across a `module` boundary specifically,
   because that is the case a reader will assume works.

And one that must **not** change: `add 1 2` still emits `add(1, 2)`, one call, no closure. Full
application is not currying and must not pay for it.

---

## 4. Leaves

### Leaf 1 — `leaf-arity-table`

Collect declared arity per function from the signatures visible in the file and in the modules
`module` loads. Reuses the existing top-level-arrow split; adds no resolution.

### Leaf 2 — `leaf-partial-application`

Emit the `move` closure for an under-applied call. Handles n-of-m for any m, which case 2 of §3
is the test for.

### Leaf 3 — `leaf-unknown-arity-diagnostic`

The §1.4 boundary: an under-applied call to a name with no signature in this file is a named
code, with a help line stating that currying needs the signature in the same file and suggesting
the explicit lambda.

### Leaf 4 — `leaf-sections-unified`

Doc 18 §4's operator sections and this row's function partial application become one mechanism
with one lowering, rather than two things that look alike. `(add 1)` and `(+ 1)` should differ in
the operator table consulted and nowhere else.

### Leaf 5 — `leaf-rust-paths-as-values`

`map String::from` — a Rust path in value position. Small, and §1.5's distinction asserted so a
reader does not conclude that Rust functions curry.

### Leaf 6 — `leaf-corpus-and-docs`

Examples exercising each §3 case; `examples/application.sdx` from plan 51 extended. Doc 18 §4's
*"without currying there is nothing to compose"* is **amended** — there is now something to
compose, which is plan 58's premise. Doc 03 §5.1 and `docs/GRAMMAR.md` updated.

---

## 5. Acceptance criteria

1. `add 1` is a closure; `add 1 2` is a call. Both asserted in a golden, so the distinction is
   visible in the generated Rust.
2. n-of-m partial application for m ≥ 3, exercised.
3. A borrowed capture produces a **rustc** error, with the message captured in `tests/ui/`.
4. `$` composes with under-application, exercised in a running example.
5. An unknown-arity under-application is a named Redox diagnostic with a `tests/ui/` fixture.
6. Operator sections and function partial application share one lowering path, asserted by a
   test that the emitted closure shape is identical for `(+ 1)` and a one-of-two `add`.
7. `map String::from` works; `String::from` under-applied is the §1.4 diagnostic, not a closure.
8. All examples green, no rustc output, stdout asserted; `construct_coverage` green.
9. Doc 18 §4's no-currying sentence is amended and the amendment cites this row.

---

## 6. Out of scope, and what it discloses

**Closure allocation becomes implicit.** `add 1` allocates where `add 1 2` does not. That is the
price of the feature and it is paid silently, which is a genuine cost in a language whose
zero-cost claim plan 44 measures by IR equality. Plan 44's harness should gain a pair for this
lowering, and that is recorded for it rather than assumed.

**Currying is per-file, bounded by §1.4.** Widening it is a resolution decision that plan 14
decision 4 already refused once.

**`String::from` is passable, not curryable.** §1.5. Rust functions have no Redox signature, so
`String::from` with too few arguments is a diagnostic rather than a closure. A reader who has
just learned that Redox functions curry will expect otherwise, which is why it is a criterion
and not only a note.

**Nullary application is untouched.** `f` with zero of one argument is not an under-application
Redox can act on, because a lone `()` is the empty argument list (plan 20 §10.6) and `f` alone is
the function item. Plan 59 takes the zero-argument binding question and plan 64 adds `\->`.

**Composition is plan 58's.** This row makes it possible: doc 18 §4 declines composition
outright, *"for lack of a lowering target: without currying there is nothing to compose"*, and
names no spelling. Plan 58 chooses `>|` / `|<`.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout asserted;
`tests/ui/` fixtures for the borrowed capture and the unknown-arity refusal; goldens re-blessed;
`construct_coverage` green.

---

## 8. Emerald reuse

None, and the absence is the point. Emerald had no currying, no sections and no juxtaposed
application; doc 02 §4.3 records its call syntax as conventional. There is nothing to transfer
and no prior experience to lean on, which is why §3 insists on asserting *where* each error
appears rather than reasoning about it.

The one transferable discipline is doc 02's, in two halves: §5.2 names the failure mode — *"a
Redox expression that reads like Rust and means something different is the worst possible failure
mode for a transpiler"* — and §8.3 is the answer, *"compiled by rustc, run, and its stdout
asserted"*. A partial application that captures the wrong argument compiles and prints a
different number, so only the run catches it.

---

## 9. Decision log

1. **Real currying, not a placeholder syntax.** `add 1 _` was the alternative — explicit, no
   implicit allocation, and it also solves the awkward-position case Haskell needs `flip` for.
   Declined because it leaves `f a b` meaning something different from what its shape says, which
   is the defect this milestone exists to remove.
2. **Capture by move, uniformly.** §1.3. The lifetime-threading option puts borrow reasoning in
   the emitter; the `Copy`-only option makes the rule depend on types Redox cannot see.
3. **Arity from the signature, not from a symbol table.** §1.2. This is what makes the row
   affordable, and the precondition is verified rather than assumed: `add2 : impl Fn(i32, i32)
   -> i32 -> i32` emits a **one**-parameter function, so the nested arrows inside a verbatim type
   are already uncounted.
4. **Bound currying to the file, explicitly.** §1.4, corrected: the first draft of this row
   claimed plan 14's `module` loads a sibling and it does not. The alternative is an
   under-applied call to a name from another file silently emitting a call and failing in rustc —
   the exact failure mode being fixed.
5. **Unify sections with partial application.** Two mechanisms producing closures that look the
   same would drift, and doc 18 §4's table would become a second specification of this row.
6. **Record the allocation cost for plan 44.** A zero-cost claim measured by IR equality should
   have a pair for the one lowering in the language that allocates.

---

## 10. Result

*Not yet executed.*
